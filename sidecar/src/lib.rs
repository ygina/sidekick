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

#[derive(Clone, PartialEq, Eq)]
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
    quack: Quack,
    log: IdentifierLog,
}

impl Sidecar {
    /// Create a new sidecar.
    pub fn new(ty: SidecarType, interface: &str, threshold: usize, bits: usize) -> Self {
        assert_eq!(bits, 32, "ERROR: <num_bits_id> must be 32");
        Self {
            ty,
            interface: interface.to_string(),
            threshold,
            bits,
            quack: Quack::new(threshold),
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
                if n != (BUFFER_SIZE as _) {
                    trace!("underfilled buffer: {} < {}", n, BUFFER_SIZE);
                    continue;
                }
                let actual_dir: Direction = addr.sll_pkttype.into();
                if actual_dir != dir {
                    trace!("packet in wrong direction: {:?}", actual_dir);
                    continue;
                }
                if addr.sll_protocol != ip_protocol {
                    trace!("not IP packet: {}", addr.sll_protocol);
                    continue;
                }
                let p = match UdpParser::parse(&buf) {
                    Some(p) => p,
                    None => {
                        trace!("not UDP packet");
                        continue;
                    }
                };
                trace!("src_mac={} dst_mac={} src_ip={} dst_ip={}, id={}",
                    p.src_mac, p.dst_mac, p.src_ip, p.dst_ip, p.identifier);
                debug!("insert {} ({:#10x})", p.identifier, p.identifier);
                // TODO: filter by QUIC connection?
                {
                    if let Some(tx) = tx.take() {
                        tx.send(()).unwrap();
                    }
                    let mut sc = sc.lock().unwrap();
                    sc.insert_packet(p.identifier);
                }
            }
        });
        Ok(rx)
    }

    /// Receive quACKs on the given UDP port. Returns the channel on which
    /// to loop received quACKs.
    pub fn listen(&self, port: u16) -> mpsc::Receiver<Quack> {
        // https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.channel.html
        // buffer up to 100 messages
        let (tx, rx) = mpsc::channel(100);
        let buf_len = {
            let quack = Quack::new(self.threshold);
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
                let quack: Quack = bincode::deserialize(&buf).unwrap();
                trace!("received quACK with count {}", quack.count);
                tx.send(quack).await.unwrap();
            }
        });
        rx
    }

    /// Snapshot the quACK.
    pub fn quack(&self) -> Quack {
        self.quack.clone()
    }

    /// Snapshot the quACK and current log.
    pub fn quack_with_log(&self) -> (Quack, IdentifierLog) {
        // TODO: don't clone the log
        (self.quack.clone(), self.log.clone())
    }
}
