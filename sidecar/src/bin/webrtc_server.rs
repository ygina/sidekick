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

use clap::Parser;
use tokio;
use tokio::net::UdpSocket;

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
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    env_logger::init();

    let args = Cli::parse();
    let sock = UdpSocket::bind(format!("0.0.0.0:{}", args.port)).await.unwrap();
    unimplemented!();
    Ok(())
}
