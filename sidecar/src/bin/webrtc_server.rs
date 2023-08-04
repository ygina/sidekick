//! Receives dummy WebRTC messages on a UDP socket.
//!
//! The first four bytes of the payload indicate a packet sequence number.
//! Store the incoming packets in a buffer and play them as soon as the next
//! packet in the sequence is available. If it ever detects a loss i.e. a
//! packet is missing after 3 later packets have been received, send a NACK
//! back to the sender that contains the sequence number of the missing packet.
//!
//! On receiving a timeout packet (sequence number is the max u32 integer),
//! print packet statistics. Print the average, p95, and p99 latencies, where
//! the latencies are how long the packet stayed in the queue. Print histogram.
use std::net::SocketAddr;
use std::collections::BinaryHeap;
use std::cmp::Reverse;

use clap::Parser;
use tokio;
use log::{trace, debug};
use tokio::net::UdpSocket;
use tokio::time::{Instant, Duration};

#[derive(Parser)]
struct Cli {
    /// Port to listen on.
    #[arg(long)]
    port: u16,
    /// Client address to send NACKs to.
    #[arg(long)]
    client_addr: SocketAddr,
    /// Number of bytes to expect in the payload.
    #[arg(long, short)]
    bytes: usize,
    /// End-to-end RTT in ms, which is also how often to resend NACKs.
    #[arg(long)]
    rtt: u64,
}

const TIMEOUT_SEQNO: u32 = u32::MAX;
const PAYLOAD_OFFSET: usize = 14 + 20 + 8;

struct Statistics {

}

impl Statistics {
    /// Create a new histogram for adding duration values.
    fn new() -> Self {
        unimplemented!()
    }

    /// Add a new duration value.
    fn add_value(&mut self, value: Duration) {
        unimplemented!()
    }

    /// Print average, p95, and p99 latency statistics.
    fn print_statistics(&self) {
        unimplemented!()
    }

    /// Print a histogram of the latency statistics.
    fn print_histogram(&self) {
        unimplemented!()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Packet {
    seqno: u32,
    time_recv: Option<Instant>,
    time_nack: Option<Instant>,
}

impl Packet {
    fn new_missing(seqno: u32) -> Self {
        Self { seqno, time_recv: None, time_nack: None }
    }

    fn new_received(seqno: u32, now: Instant) -> Self {
        Self { seqno, time_recv: Some(now), time_nack: None }
    }
}

struct BufferedPackets {
    nack_frequency: Duration,
    next_seqno: u32,
}

impl BufferedPackets {
    fn new(nack_frequency: Duration) -> Self {
        Self {
            nack_frequency,
            next_seqno: 0,
        }
    }

    /// Receive a packet with this sequence number.
    fn recv_seqno(&mut self, seqno: u32, now: Instant) {
        unimplemented!()
    }

    /// Return the received time of the next packet to play if the next packet
    /// in the sequence is available. Removes that packet from the buffer.
    fn pop_seqno(&mut self) -> Option<Instant> {
        unimplemented!()
    }

    /// Send NACKs to the given client address if any packets are missing i.e.,
    /// three later packets have been received. Also resend NACKs if it has
    /// been more than an RTT since the last NACK for that sequence number.
    /// It may be considerably more than an RTT for NACK retransmissions if
    /// this function is only called on receiving a packet.
    fn send_nacks(&mut self, addr: SocketAddr) {
        unimplemented!()
    }
}


#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let args = Cli::parse();
    let mut stats = Statistics::new();

    // Listen for incoming packets.
    let nack_frequency = Duration::from_millis(args.rtt);
    let mut pkts = BufferedPackets::new(nack_frequency);
    let mut buf = Vec::new();
    let sock = UdpSocket::bind(format!("0.0.0.0:{}", args.port)).await.unwrap();
    debug!("webrtc server is now listening");
    loop {
        let (len, _addr) = sock.recv_from(&mut buf).await?;
        let seqno = u32::from_be_bytes([
            buf[PAYLOAD_OFFSET],
            buf[PAYLOAD_OFFSET + 1],
            buf[PAYLOAD_OFFSET + 2],
            buf[PAYLOAD_OFFSET + 3],
        ]);
        trace!("received seqno {} ({} bytes)", seqno, len);
        if seqno == TIMEOUT_SEQNO {
            debug!("timeout message received");
            break;
        }
        let now = Instant::now();
        pkts.recv_seqno(seqno, now);
        while let Some(time_recv) = pkts.pop_seqno() {
            stats.add_value(now - time_recv);
        }
        pkts.send_nacks(args.client_addr);
    }

    // Print statistics before exiting.
    stats.print_statistics();
    stats.print_histogram();
    Ok(())
}
