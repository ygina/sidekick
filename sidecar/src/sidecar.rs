use std::sync::{Arc, Mutex};
use quack::*;
use log::{trace, debug, info};
use tokio;
use tokio::sync::oneshot;

use crate::{Quack, Socket};
use crate::socket::SockAddr;
use crate::buffer::{BUFFER_SIZE, Direction, UdpParser};

#[derive(Clone)]
pub struct Sidecar {
    pub interface: String,
    pub threshold: usize,
    pub bits: usize,
    #[cfg(feature = "benchmark")]
    pub start_time: Option<tokio::time::Instant>,
    quack: PowerSumQuack<u32>,
    log: Vec<u32>,
}

impl Sidecar {
    /// Create a new sidecar.
    pub fn new(interface: &str, threshold: usize, bits: usize) -> Self {
        assert_eq!(bits, 32, "ERROR: <num_bits_id> must be 32");
        Self {
            interface: interface.to_string(),
            threshold,
            bits,
            #[cfg(feature = "benchmark")]
            start_time: None,
            quack: PowerSumQuack::new(threshold),
            log: vec![],
        }
    }

    /// Insert a packet into the cumulative quACK. Should be used by quACK
    /// receivers, such as in the client code, with direct access to sent
    /// packets. Typically if this function is used, do not call start().
    pub fn insert_packet(&mut self, id: u32) {
        if self.threshold != 0 {
            self.quack.insert(id);
        }
    }

    /// Reset the sidecar state.
    pub fn reset(&mut self) {
        self.quack = PowerSumQuack::new(self.threshold);
        self.log = vec![];
    }

    /// Start the raw socket that listens to the specified interface and
    /// accumulates those packets in a quACK. If the sidecar is a quACK sender,
    /// only listens for incoming packets. If the sidecar is a quACK receiver,
    /// only listens for outgoing packets, and additionally logs the packet
    /// identifiers.
    /// Returns a channel that indicates when the first packet is sniffed.
    pub fn start(
        sc: Arc<Mutex<Sidecar>>,
        my_ipv4_addr: [u8; 4],
    ) -> Result<oneshot::Receiver<()>, String> {
        let interface = sc.lock().unwrap().interface.clone();
        let sock = Socket::new(interface.clone())?;
        sock.set_promiscuous()?;

        // Creates the channel that indicates when the first packet is sniffed.
        let (tx, rx) = oneshot::channel();

        // Loop over received packets
        tokio::task::spawn_blocking(move || {
            info!("tapping socket on fd={} interface={}", sock.fd, interface);
            let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
            let mut addr = SockAddr::new_sockaddr_ll();
            let ip_protocol = (libc::ETH_P_IP as u16).to_be();
            let mut tx = Some(tx);
            loop {
                let n = match sock.recvfrom(&mut addr, &mut buf) {
                    Ok(n) => n,
                    Err(_) => { break; },
                };
                trace!("received {} bytes: {:?}", n, buf);
                if Direction::Incoming != addr.sll_pkttype.into() {
                    continue;
                }
                if addr.sll_protocol != ip_protocol {
                    trace!("not IP packet: {}", addr.sll_protocol);
                    continue;
                }
                if !UdpParser::is_udp(&buf) {
                    trace!("not UDP packet");
                    continue;
                }

                // Reset the quack if the dst IP is our own (and not for
                // another e2e quic connection).
                if UdpParser::parse_dst_ip(&buf) == my_ipv4_addr {
                    // TODO: check if dst port corresponds to this connection
                    sc.lock().unwrap().reset();
                    continue;
                }

                // Otherwise parse the identifier and insert it into the quack.
                if n != (BUFFER_SIZE as _) {
                    trace!("underfilled buffer: {} < {}", n, BUFFER_SIZE);
                    continue;
                }
                let id = UdpParser::parse_identifier(&buf);
                debug!("insert {} ({:#10x})", id, id);
                // TODO: filter by QUIC connection?
                {
                    let mut sc = sc.lock().unwrap();
                    if let Some(tx) = tx.take() {
                        tx.send(()).unwrap();
                        #[cfg(feature = "benchmark")]
                        {
                            sc.start_time = Some(tokio::time::Instant::now());
                        }
                    }
                    sc.insert_packet(id);
                    #[cfg(feature = "quack_log")]
                    println!("quack {:?} {} {}", std::time::Instant::now(), id, sc.quack.count());
                }
            }
        });
        Ok(rx)
    }

    /// Start the raw socket that listens to the specified interface and
    /// accumulates those packets in a quACK. If the sidecar is a quACK sender,
    /// only listens for incoming packets. If the sidecar is a quACK receiver,
    /// only listens for outgoing packets, and additionally logs the packet
    /// identifiers.
    /// Returns a channel that indicates when the first packet is sniffed.
    pub async fn start_frequency_pkts(
        &mut self,
        _frequency_pkts: usize,
        _sendaddr: std::net::SocketAddr,
    ) -> Result<(), String> {
        /*
        let recvsock = Socket::new(self.interface.clone())?;
        let sendsock = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        recvsock.set_promiscuous()?;

        // Loop over received packets
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        info!("tapping socket on fd={} interface={}", recvsock.fd, self.interface);
        let mut addr = SockAddr::new_sockaddr_ll();
        let ip_protocol = (libc::ETH_P_IP as u16).to_be();
        let mut mod_count = 0;
        loop {
            // TODO: resets, dont' duplicate code from start()
            let n = recvsock.recvfrom(&mut addr, &mut buf).unwrap();
            trace!("received {} bytes: {:?}", n, buf);
            if n != (BUFFER_SIZE as _) {
                trace!("underfilled buffer: {} < {}", n, BUFFER_SIZE);
                continue;
            }
            let actual_dir: Direction = addr.sll_pkttype.into();
            if actual_dir != Direction::Incoming {
                trace!("packet in wrong direction: {:?}", actual_dir);
                continue;
            }
            if addr.sll_protocol != ip_protocol {
                trace!("not IP packet: {}", addr.sll_protocol);
                continue;
            }
            if !UdpParser::is_udp(&buf) {
                trace!("not UDP packet");
                continue;
            }
            let id = UdpParser::parse_identifier(&buf);
            debug!("insert {} ({:#10x})", id, id);
            // TODO: filter by QUIC connection?
            self.quack.insert(id);
            #[cfg(feature = "quack_log")]
            println!("quack {:?} {} {}", std::time::Instant::now(), id, self.quack.count());
            mod_count = (mod_count + 1) % frequency_pkts;
            if mod_count == 0 {
                let bytes = bincode::serialize(&self.quack).unwrap();
                info!("quack {}", self.quack.count());
                sendsock.send_to(&bytes, sendaddr).await.unwrap();
            }
        }
        */
        unimplemented!()
    }

    /// Snapshot the quACK.
    pub fn quack(&self) -> PowerSumQuack<u32> {
        self.quack.clone()
    }

    /// Snapshot the quACK and current log.
    pub fn quack_with_log(&self) -> (PowerSumQuack<u32>, Vec<u32>) {
        // TODO: don't clone the log
        (self.quack.clone(), self.log.clone())
    }
}
