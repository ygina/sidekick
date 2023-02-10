use std::sync::{Arc, Mutex};
use quack::*;
use bincode;
use log::{trace, debug, info};
use tokio;
use tokio::{sync::{mpsc, oneshot}, net::UdpSocket};

mod socket;
mod buffer;

pub use socket::Socket;
use socket::SockAddr;
use buffer::{BUFFER_SIZE, Direction, UdpParser};
pub use buffer::ID_OFFSET;
use crate::Quack;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SidecarType {
    QuackSender,
    QuackReceiver,
}

#[derive(Clone)]
pub struct Sidecar {
    pub ty: SidecarType,
    pub interface: String,
    pub threshold: usize,
    pub bits: usize,
    // TODO: is there a better way to do synchronization?
    quack: PowerSumQuack,
    log: IdentifierLog,
}

impl Sidecar {
    /// Create a new sidecar.
    pub fn new(
        ty: SidecarType,
        interface: &str,
        threshold: usize,
        bits: usize,
    ) -> Self {
        assert_eq!(bits, 32, "ERROR: <num_bits_id> must be 32");
        Self {
            ty,
            interface: interface.to_string(),
            threshold,
            bits,
            quack: PowerSumQuack::new(threshold),
            log: vec![],
        }
    }

    /// Insert a packet into the cumulative quACK. Should be used by quACK
    /// receivers, such as in the client code, with direct access to sent
    /// packets. Typically if this function is used, do not call start().
    pub fn insert_packet(&mut self, id: Identifier) {
        self.quack.insert(id);
        if self.ty == SidecarType::QuackReceiver {
            self.log.push(id);
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
    ) -> Result<oneshot::Receiver<()>, String> {
        let (interface, ty) = {
            let sc = sc.lock().unwrap();
            (sc.interface.clone(), sc.ty.clone())
        };
        let sock = Socket::new(interface.clone())?;
        sock.set_promiscuous()?;

        // Creates the channel that indicates when the first packet is sniffed.
        let (tx, rx) = oneshot::channel();

        // Loop over received packets
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        tokio::task::spawn_blocking(move || {
            info!("tapping socket on fd={} interface={}", sock.fd, interface);
            let mut addr = SockAddr::new_sockaddr_ll();
            let ip_protocol = (libc::ETH_P_IP as u16).to_be();
            let dir = match ty {
                SidecarType::QuackSender => Direction::Incoming,
                SidecarType::QuackReceiver => Direction::Outgoing,
            };
            let mut tx = Some(tx);
            loop {
                let n = sock.recvfrom(&mut addr, &mut buf).unwrap();
                trace!("received {} bytes: {:?}", n, buf);
                let actual_dir: Direction = addr.sll_pkttype.into();
                if actual_dir != dir {
                    trace!("packet in wrong direction: {:?}", actual_dir);
                    continue;
                }
                if addr.sll_protocol != ip_protocol {
                    trace!("not IP packet: {}", addr.sll_protocol);
                    continue;
                }

                // Reset the quack if the dst IP is our own (and not for
                // another e2e quic connection).
                let dst_ip = match UdpParser::parse_dst_ip(&buf) {
                    Some(dst_ip) => dst_ip,
                    None => {
                        trace!("not UDP packet");
                        continue;
                    }
                };
                if dst_ip == [10, 0, 2, 1] {
                    // TODO: check if dst port corresponds to this connection
                    sc.lock().unwrap().reset();
                    continue;
                }

                // Otherwise parse the identifier and insert it into the quack.
                if n != (BUFFER_SIZE as _) {
                    trace!("underfilled buffer: {} < {}", n, BUFFER_SIZE);
                    continue;
                }
                let id = UdpParser::parse_identifier(&buf).unwrap();
                debug!("insert {} ({:#10x})", id, id);
                // TODO: filter by QUIC connection?
                {
                    if let Some(tx) = tx.take() {
                        tx.send(()).unwrap();
                    }
                    let mut sc = sc.lock().unwrap();
                    #[cfg(feature = "quack_log")]
                    println!("quack {:?} {}", std::time::Instant::now(), id);
                    // TODO: sc.insert_packet(id);
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
        frequency_pkts: usize,
        sendaddr: std::net::SocketAddr,
    ) -> Result<(), String> {
        let recvsock = Socket::new(self.interface.clone())?;
        let sendsock = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        recvsock.set_promiscuous()?;

        // Loop over received packets
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        info!("tapping socket on fd={} interface={}", recvsock.fd, self.interface);
        let mut addr = SockAddr::new_sockaddr_ll();
        let ip_protocol = (libc::ETH_P_IP as u16).to_be();
        assert_eq!(self.ty, SidecarType::QuackSender);
        let mut mod_count = 0;
        loop {
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
            let id = match UdpParser::parse_identifier(&buf) {
                Some(id) => id,
                None => {
                    trace!("not UDP idacket");
                    continue;
                }
            };
            debug!("insert {} ({:#10x})", id, id);
            // TODO: filter by QUIC connection?
            // TODO: self.quack.insert(id);
            #[cfg(feature = "quack_log")]
            println!("quack {:?} {}", std::time::Instant::now(), id);
            mod_count = (mod_count + 1) % frequency_pkts;
            if mod_count == 0 {
                let bytes = bincode::serialize(&self.quack).unwrap();
                info!("quack {}", self.quack.count());
                sendsock.send_to(&bytes, sendaddr).await.unwrap();
            }
        }
    }

    /// Receive quACKs on the given UDP port. Returns the channel on which
    /// to loop received quACKs.
    pub fn listen(&self, port: u16) -> mpsc::Receiver<PowerSumQuack> {
        // https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.channel.html
        // buffer up to 100 messages
        let (tx, rx) = mpsc::channel(100);
        let buf_len = {
            let quack = PowerSumQuack::new(self.threshold);
            bincode::serialize(&quack).unwrap().len()
        };
        debug!("allocating {} bytes for receiving quACKs", buf_len);
        tokio::spawn(async move {
            let addr = format!("0.0.0.0:{}", port);
            info!("listening for quACKs on {}", addr);
            let socket = UdpSocket::bind(addr).await.unwrap();
            let mut buf = vec![0; buf_len];
            loop {
                let (nbytes, _) = socket.recv_from(&mut buf).await.unwrap();
                assert_eq!(nbytes, buf.len());
                // TODO: check that it's actually a quack
                let quack: PowerSumQuack = bincode::deserialize(&buf).unwrap();
                trace!("received quACK with count {}", quack.count());
                tx.send(quack).await.unwrap();
            }
        });
        rx
    }

    /// Snapshot the quACK.
    pub fn quack(&self) -> PowerSumQuack {
        self.quack.clone()
    }

    /// Snapshot the quACK and current log.
    pub fn quack_with_log(&self) -> (PowerSumQuack, IdentifierLog) {
        // TODO: don't clone the log
        (self.quack.clone(), self.log.clone())
    }
}
