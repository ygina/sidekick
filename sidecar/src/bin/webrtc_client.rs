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
use std::io;
use std::net::SocketAddr;

use clap::{ValueEnum, Parser};
use tokio;
use tokio::net::UdpSocket;
use tokio::time::{Instant, Duration};

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

const NACK_BUFFER_SIZE: usize = 4;

struct PacketSender {
    addr: SocketAddr,
    payload: Vec<u8>,
    send_sock: UdpSocket,
}

impl PacketSender {
    async fn new(server_addr: &SocketAddr, bytes: usize) -> io::Result<Self> {
        Ok(Self {
            addr: server_addr.clone(),
            payload: vec![0xFF; bytes],
            send_sock: UdpSocket::bind("0.0.0.0:0").await?,
        })
    }

    /// Send a packet with this sequence number to the server.
    async fn send(&mut self, seqno: u32) -> io::Result<()> {
        let seqno = seqno.to_be_bytes();
        self.payload[0] = seqno[0];
        self.payload[1] = seqno[1];
        self.payload[2] = seqno[2];
        self.payload[3] = seqno[3];
        self.send_sock.send_to(&self.payload, &self.addr).await?;
        Ok(())
    }
}

/// Spawn a thread that listens for end-to-end NACKs and retransmit packets
/// when requested.
fn listen_for_nacks(mut sender: PacketSender, port: u16) {
    let mut buf: [u8; NACK_BUFFER_SIZE] = [0; NACK_BUFFER_SIZE];
    tokio::spawn(async move {
        let sock = UdpSocket::bind(format!("0.0.0.0:{}", port)).await.unwrap();
        loop {
            let (len, _addr) = sock.recv_from(&mut buf).await.unwrap();
            assert_eq!(len, NACK_BUFFER_SIZE);
            let seqno = u32::from_be_bytes([
                buf[0],
                buf[1],
                buf[2],
                buf[3],
            ]);
            sender.send(seqno).await.unwrap();
        }
    });
}

/// Spawn a thread that listens for sidecar quACKs and retransmit packets when
/// determined missing.
fn listen_for_quacks(
    mut sender: PacketSender, quack_style: QuackStyle, quack_port: u16,
    reset_addr: SocketAddr, threshold: usize,
) {
    unimplemented!()
}

/// Send a stream of packets at the specified frequency with the given payload.
/// When the timeout is reached, send several timeout packets and return.
async fn stream_data(
    mut sender: PacketSender, timeout: Duration, frequency: Duration,
) -> io::Result<()> {
    let mut interval = tokio::time::interval(frequency);
    let start = Instant::now();

    // Send packets with increasing sequence numbers until the elapsed time
    // is greater than the timeout.
    for seqno in 0..u32::MAX {
        interval.tick().await;
        sender.send(seqno).await?;
        if Instant::now() - start > timeout {
            break;
        }
    }

    // Send the timeout message.
    sender.send(u32::MAX).await?;
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    env_logger::init();

    let args = Cli::parse();
    listen_for_nacks(
        PacketSender::new(&args.server_addr, args.bytes).await?,
        args.port,
    );
    if let Some(quack_style) = args.quack_style {
        listen_for_quacks(
            PacketSender::new(&args.server_addr, args.bytes).await?,
            quack_style,
            args.quack_port,
            args.reset_addr,
            args.threshold,
        );
    }
    stream_data(
        PacketSender::new(&args.server_addr, args.bytes).await?,
        Duration::from_secs(args.timeout),
        Duration::from_millis(args.frequency),
    ).await?;
    Ok(())
}
