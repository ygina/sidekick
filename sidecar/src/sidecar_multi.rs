use std::sync::{Arc, Mutex};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::collections::HashMap;

use quack::*;
use tokio;
use tokio::{sync::oneshot, time::Instant};
use log::trace;

use crate::{Quack, Socket};
use crate::socket::SockAddr;
use crate::buffer::{BUFFER_SIZE, Direction, UdpParser};

const SIDECAR_IP_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 2, 1));

const IP_PROTOCOL: u16 = (libc::ETH_P_IP as u16).to_be();

#[derive(Clone)]
pub struct SidecarMulti {
    /// Interface to listen on
    pub interface: String,

    /// Quack properties
    pub threshold: usize,
    pub bits: usize,

    /// Time the first packet is inserted, for benchmarking
    #[cfg(feature = "benchmark")]
    pub start_time: Option<Instant>,

    /// Map from UDP source and dest address to the quack
    senders: HashMap<(SocketAddr, SocketAddr), PowerSumQuack<u32>>,
}

enum Action {
    Skip,
    Reset {
        src_addr: SocketAddr,
        dst_addr: SocketAddr,
    },
    Insert {
        src_addr: SocketAddr,
        dst_addr: SocketAddr,
        sidecar_id: u32,
    },
}

impl SidecarMulti {
    /// Create a new sidecar.
    pub fn new(interface: &str, threshold: usize, bits: usize) -> Self {
        assert_eq!(bits, 32, "ERROR: <num_bits_id> must be 32");
        Self {
            interface: interface.to_string(),
            threshold,
            bits,
            #[cfg(feature = "benchmark")]
            start_time: None,
            senders: HashMap::new(),
        }
    }

    pub fn reset(&mut self, src_dst: &(SocketAddr, SocketAddr)) {
        self.senders
            .get_mut(src_dst)
            .map(|quack| *quack = PowerSumQuack::new(self.threshold));
    }

    pub fn insert(
        &mut self, src_dst: (SocketAddr, SocketAddr), sidecar_id: u32,
    ) {
        self.senders
            .entry(src_dst)
            .or_insert(PowerSumQuack::new(self.threshold))
            .insert(sidecar_id);
    }

    pub fn quack(
        &self, src_dst: &(SocketAddr, SocketAddr),
    ) -> Option<PowerSumQuack<u32>> {
        self.senders.get(src_dst).map(|quack| quack.clone())
    }

    pub fn senders(&self) -> &HashMap<(SocketAddr, SocketAddr), PowerSumQuack<u32>> {
        &self.senders
    }
}

fn process_one_packet(
    n: isize, buf: &[u8; BUFFER_SIZE], addr: &libc::sockaddr_ll,
) -> Action {
    if Direction::Incoming != addr.sll_pkttype.into() {
        return Action::Skip;
    }
    if addr.sll_protocol != IP_PROTOCOL {
        return Action::Skip;
    }
    if !UdpParser::is_udp(buf) {
        return Action::Skip;
    }

    // Reset the quack if the dst IP is our own (and not for another e2e quic
    // connection).
    let src_addr = UdpParser::parse_src_addr(buf);
    let dst_addr = UdpParser::parse_dst_addr(buf);
    if dst_addr.ip() == SIDECAR_IP_ADDR {  // TODO: check dst port
        return Action::Reset { src_addr, dst_addr };
    }

    // Otherwise parse the identifier and insert it into the quack.
    if n != (BUFFER_SIZE as _) {
        return Action::Skip;
    }
    let sidecar_id = UdpParser::parse_identifier(&buf);
    Action::Insert { src_addr, dst_addr, sidecar_id }
}

/// Start the raw socket that listens to the specified interface. Creates a new
/// quack for every source socket address and accumulates the packets for that
/// connection. Returns a channel that indicates the start time of when the
/// first packet is sniffed.
pub fn start_sidecar_multi(
    sc: Arc<Mutex<SidecarMulti>>,
) -> Result<oneshot::Receiver<Instant>, String> {
    let interface = sc.lock().unwrap().interface.clone();
    let sock = Socket::new(interface.clone())?;
    sock.set_promiscuous()?;

    // Creates the channel that indicates the time of when the first packet is
    // sniffed and inserted into a quack
    let (tx, rx) = oneshot::channel();
    tokio::task::spawn_blocking(move || {
        let mut addr = SockAddr::new_sockaddr_ll();
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let mut tx = Some(tx);

        loop {
            let n = sock.recvfrom(&mut addr, &mut buf).unwrap();
            trace!("received {} bytes: {:?}", n, buf);
            match process_one_packet(n, &buf, &addr) {
                Action::Skip => { continue; }
                Action::Reset { src_addr, dst_addr } => {
                    sc.lock().unwrap().reset(&(src_addr, dst_addr));
                }
                Action::Insert { src_addr, dst_addr, sidecar_id } => {
                    let mut sc = sc.lock().unwrap();
                    if let Some(tx) = tx.take() {
                        let now = Instant::now();
                        tx.send(now).unwrap();
                        #[cfg(feature = "benchmark")]
                        {
                            sc.start_time = Some(now);
                        }
                    }
                    sc.insert((src_addr, dst_addr), sidecar_id);
                }
            }
        }
    });
    Ok(rx)
}
