use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use log::{info, trace};
use quack::*;
use tokio;
use tokio::{net::UdpSocket, sync::oneshot, time::Instant};

use crate::buffer::{Direction, UdpParser, BUFFER_SIZE};
use crate::socket::SockAddr;
use crate::{Quack, Socket};

type AddrKey = [u8; 12];

const IP_PROTOCOL: u16 = (libc::ETH_P_IP as u16).to_be();

#[cfg(any(feature = "cycles", feature = "cycles_summary"))]
static mut CYCLES_COUNT: u64 = 0;
#[cfg(any(feature = "cycles", feature = "cycles_summary"))]
static mut CYCLES: [u64; 5] = [0; 5];

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

    pub fn insert(&mut self, addr_key: AddrKey, sidecar_id: u32) -> &PowerSumQuack<u32> {
        // ***CYCLES START step 2 hash address key
        #[cfg(feature = "cycles")]
        let start2 = unsafe { core::arch::x86_64::_rdtsc() };
        let entry = self
            .senders
            .entry(addr_key)
            .or_insert(PowerSumQuack::new(self.threshold));
        // ***CYCLES STOP step 2 hash address key
        #[cfg(feature = "cycles")]
        unsafe {
            let stop2 = core::arch::x86_64::_rdtsc();
            CYCLES[2] += stop2 - start2;
        }
        // ***CYCLES START step 4 insert id into quack
        #[cfg(feature = "cycles")]
        let start4 = unsafe { core::arch::x86_64::_rdtsc() };
        entry.insert(sidecar_id);
        // ***CYCLES STOP step 4 insert id into quack
        #[cfg(feature = "cycles")]
        unsafe {
            let stop4 = core::arch::x86_64::_rdtsc();
            CYCLES[4] += stop4 - start4;
        }
        self.senders.get(&addr_key).as_ref().unwrap()
    }

    pub fn quack(&self, addr_key: &AddrKey) -> Option<PowerSumQuack<u32>> {
        self.senders.get(addr_key).map(|quack| quack.clone())
    }

    pub fn senders(&self) -> &HashMap<AddrKey, PowerSumQuack<u32>> {
        &self.senders
    }
}

fn process_one_packet(
    n: isize,
    buf: &[u8; BUFFER_SIZE],
    addr: &libc::sockaddr_ll,
    my_addr: [u8; 6],
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
    if &addr_key[6..12] == my_addr {
        return Action::Reset { addr_key };
    }

    // Otherwise parse the identifier and insert it into the quack.
    if n != (BUFFER_SIZE as _) {
        return Action::Skip;
    }

    // ***CYCLES START step 3 parse identifier
    #[cfg(feature = "cycles")]
    let start3 = unsafe { core::arch::x86_64::_rdtsc() };
    let sidecar_id = UdpParser::parse_identifier(&buf);
    // ***CYCLES STOP step 3 parse identifier
    #[cfg(feature = "cycles")]
    unsafe {
        let stop3 = core::arch::x86_64::_rdtsc();
        CYCLES[3] += stop3 - start3;
    }
    Action::Insert {
        addr_key,
        sidecar_id,
    }
}

#[cfg(any(feature = "cycles", feature = "cycles_summary"))]
unsafe fn print_cycles_count_summary() {
    CYCLES_COUNT += 1;
    if CYCLES_COUNT % 1000 == 0 {
        println!(
            "{:?}",
            CYCLES
                .clone()
                .into_iter()
                .map(|cycles| cycles / CYCLES_COUNT)
                .collect::<Vec<_>>()
        );
    }
}

/// Start the raw socket that listens to the specified interface. Creates a new
/// quack for every source socket address and accumulates the packets for that
/// connection. Returns a channel that indicates the start time of when the
/// first packet is sniffed.
pub fn start_sidecar_multi(
    sc: Arc<Mutex<SidecarMulti>>,
    my_addr: [u8; 6],
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
            // ***CYCLES START step 0 total
            #[cfg(feature = "cycles_summary")]
            let start0 = unsafe { core::arch::x86_64::_rdtsc() };
            // ***CYCLES START step 1 sniff packet
            #[cfg(feature = "cycles")]
            let start1 = unsafe { core::arch::x86_64::_rdtsc() };
            let n = sock.recvfrom(&mut addr, &mut buf).unwrap();
            // ***CYCLES STOP step 1 sniff packet
            #[cfg(feature = "cycles")]
            let stop1 = unsafe { core::arch::x86_64::_rdtsc() };
            trace!("received {} bytes: {:?}", n, buf);
            match process_one_packet(n, &buf, &addr, my_addr) {
                Action::Skip => {
                    continue;
                }
                Action::Reset { addr_key } => {
                    info!("resetting quacks {:?}", addr_key);
                    sc.lock().unwrap().senders = HashMap::new();
                }
                Action::Insert {
                    addr_key,
                    sidecar_id,
                } => {
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
            // ***CYCLES STOP step 0 total
            #[cfg(feature = "cycles_summary")]
            unsafe {
                let stop0 = core::arch::x86_64::_rdtsc();
                CYCLES[0] += stop0 - start0;
                print_cycles_count_summary();
            }
            #[cfg(feature = "cycles")]
            unsafe {
                CYCLES[1] += stop1 - start1;
                print_cycles_count_summary();
            }
        }
    });
    Ok(rx)
}

pub async fn start_sidecar_multi_frequency_pkts(
    sc: Arc<Mutex<SidecarMulti>>,
    my_addr: [u8; 6],
    frequency_pkts: u32,
    sendaddr: std::net::SocketAddr,
) -> Result<(), String> {
    let interface = sc.lock().unwrap().interface.clone();
    let sock = Socket::new(interface.clone())?;
    sock.set_promiscuous()?;

    // Creates the channel that indicates the time of when the first packet is
    // sniffed and inserted into a quack
    let mut addr = SockAddr::new_sockaddr_ll();
    let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    let sendsock = UdpSocket::bind("0.0.0.0:0").await.unwrap();

    loop {
        let n = sock.recvfrom(&mut addr, &mut buf).unwrap();
        trace!("received {} bytes: {:?}", n, buf);
        match process_one_packet(n, &buf, &addr, my_addr) {
            Action::Skip => {
                continue;
            }
            Action::Reset { addr_key } => {
                info!("resetting quacks {:?}", addr_key);
                sc.lock().unwrap().senders = HashMap::new();
            }
            Action::Insert {
                addr_key,
                sidecar_id,
            } => {
                let mut sc = sc.lock().unwrap();
                let quack = sc.insert(addr_key, sidecar_id);
                if quack.count() % frequency_pkts == 0 {
                    let bytes = bincode::serialize(&quack).unwrap();
                    trace!("quack {} {:?}", quack.count(), addr_key);
                    sendsock.send_to(&bytes, sendaddr).await.unwrap();
                }
            }
        }
    }
}
