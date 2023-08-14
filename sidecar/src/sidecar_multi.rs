use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use quack::*;
use tokio;
use tokio::{sync::oneshot, time::Instant};
use log::trace;

use crate::{Quack, Socket};
use crate::socket::SockAddr;
use crate::buffer::{BUFFER_SIZE, Direction, UdpParser};

type AddrKey = [u8; 12];

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
    senders: HashMap<AddrKey, PowerSumQuack<u32>>,
}

enum Action {
    Skip,
    Reset { addr_key: AddrKey },
    Insert { addr_key: AddrKey, sidecar_id: u32 },
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

    pub fn reset(&mut self, addr_key: &AddrKey) {
        self.senders
            .get_mut(addr_key)
            .map(|quack| *quack = PowerSumQuack::new(self.threshold));
    }

    pub fn insert(
        &mut self, addr_key: AddrKey, sidecar_id: u32,
    ) {
        self.senders
            .entry(addr_key)
            .or_insert(PowerSumQuack::new(self.threshold))
            .insert(sidecar_id);
    }

    pub fn quack(
        &self, addr_key: &AddrKey,
    ) -> Option<PowerSumQuack<u32>> {
        self.senders.get(addr_key).map(|quack| quack.clone())
    }

    pub fn senders(&self) -> &HashMap<AddrKey, PowerSumQuack<u32>> {
        &self.senders
    }
}

fn process_one_packet(
    n: isize, buf: &[u8; BUFFER_SIZE], addr: &libc::sockaddr_ll,
    my_ipv4_addr: [u8; 4],
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
    let addr_key = UdpParser::parse_addr_key(buf);
    if &addr_key[6..10] == my_ipv4_addr {
        return Action::Reset { addr_key };
    }

    // Otherwise parse the identifier and insert it into the quack.
    if n != (BUFFER_SIZE as _) {
        return Action::Skip;
    }
    let sidecar_id = UdpParser::parse_identifier(&buf);
    Action::Insert { addr_key, sidecar_id }
}

/// Start the raw socket that listens to the specified interface. Creates a new
/// quack for every source socket address and accumulates the packets for that
/// connection. Returns a channel that indicates the start time of when the
/// first packet is sniffed.
pub fn start_sidecar_multi(
    sc: Arc<Mutex<SidecarMulti>>,
    my_ipv4_addr: [u8; 4],
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
            match process_one_packet(n, &buf, &addr, my_ipv4_addr) {
                Action::Skip => { continue; }
                Action::Reset { addr_key } => {
                    sc.lock().unwrap().reset(&addr_key);
                }
                Action::Insert { addr_key, sidecar_id } => {
                    let mut sc = sc.lock().unwrap();
                    if let Some(tx) = tx.take() {
                        let now = Instant::now();
                        tx.send(now).unwrap();
                        #[cfg(feature = "benchmark")]
                        {
                            sc.start_time = Some(now);
                        }
                    }
                    sc.insert(addr_key, sidecar_id);
                }
            }
        }
    });
    Ok(rx)
}
