//! Send dummy WebRTC messages to a UDP socket.
//!
//! The first four bytes of the payload indicate a packet sequence number.
//! Send a packet every <FREQUENCY> milliseconds, containing <BYTES> bytes of
//! dummy data. (Sending a 240-byte payload every 20ms represents a 96 kbps
//! stream.) When <TIMEOUT> time has elapsed, send a timeout packet where the
//! sequence number is the max u32 integer. On receiving a NACK, retransmit
//! the missing packet that was identified in the NACK.
//!
//! When using a quACK, immediately retransmit missing packets from the quACK
//! i.e. a packet is missing after 3 later packets have been received. If the
//! quACK is undecodeable, send a reset message to the socket address from
//! which the quACK was sent.
use std::net::SocketAddr;

use clap::{ValueEnum, Parser};
use tokio;
use tokio::net::UdpSocket;

#[derive(ValueEnum, PartialEq, Debug, Clone, Copy)]
#[clap(rename_all = "snake_case")]
enum QuackStyle {
    StrawmanA,
    StrawmanB,
    StrawmanC,
    PowerSum,
}

#[derive(Parser)]
struct Cli {
    /// Server address to send dummy WebRTC messages to.
    #[arg(long)]
    server_addr: SocketAddr,
    /// Port to listen on for NACKs.
    #[arg(long)]
    port: u16,
    /// Number of seconds to stream data before sending a timeout message.
    #[arg(long, short)]
    timeout: usize,
    /// Number of bytes to send in the payload, including the sequence number.
    #[arg(long, short)]
    bytes: usize,
    /// Frequency at which to send packets, in milliseconds.
    #[arg(long, short)]
    frequency: usize,
    /// Style of quack to expect.
    #[arg(long, value_enum)]
    quack_style: Option<QuackStyle>,
    /// Port to listen on for quACKs.
    #[arg(long, default_value_t = 5103)]
    quack_port: u16,
    /// QuACK threshold.
    #[arg(long, default_value_t = 5)]
    threshold: usize,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    env_logger::init();

    let args = Cli::parse();
    let sock = UdpSocket::bind(format!("0.0.0.0:{}", args.port)).await.unwrap();
    unimplemented!();
    Ok(())
}
