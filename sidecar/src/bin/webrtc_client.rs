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
use tokio::time::Duration;

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
    timeout: u64,
    /// Number of bytes to send in the payload, including the sequence number.
    #[arg(long, short)]
    bytes: usize,
    /// Frequency at which to send packets, in milliseconds.
    #[arg(long, short)]
    frequency: u64,
    /// Style of quack to expect.
    #[arg(long, value_enum)]
    quack_style: Option<QuackStyle>,
    /// Port to listen on for quACKs.
    #[arg(long, default_value_t = 5103)]
    quack_port: u16,
    /// Address to send quACK resets too.
    #[arg(long, default_value = "10.0.2.1:1234")]
    reset_addr: SocketAddr,
    /// QuACK threshold.
    #[arg(long, default_value_t = 5)]
    threshold: usize,
}

/// Spawn a thread that listens for end-to-end NACKs and retransmit packets
/// when requested.
fn listen_for_nacks(port: u16) {
    unimplemented!()
}

/// Spawn a thread that listens for sidecar quACKs and retransmit packets when
/// determined missing.
fn listen_for_quacks(
    quack_style: QuackStyle, quack_port: u16, reset_addr: SocketAddr,
    threshold: usize,
) {
    unimplemented!()
}

/// Send a stream of packets at the specified frequency with the given payload.
/// When the timeout is reached, send several timeout packets and return.
async fn stream_data(
    server_addr: SocketAddr, timeout: Duration, bytes: usize,
    frequency: Duration,
) {
    unimplemented!()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    env_logger::init();

    let args = Cli::parse();
    listen_for_nacks(args.port);
    if let Some(quack_style) = args.quack_style {
        listen_for_quacks(
            quack_style,
            args.quack_port,
            args.reset_addr,
            args.threshold,
        );
    }
    stream_data(
        args.server_addr,
        Duration::from_secs(args.timeout),
        args.bytes,
        Duration::from_millis(args.frequency),
    ).await;
    Ok(())
}
