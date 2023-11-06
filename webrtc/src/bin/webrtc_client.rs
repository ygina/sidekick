//! Send dummy WebRTC messages to a UDP socket.
//!
//! The first four bytes of the payload indicate a packet sequence number.
//! The sequence numbers start at 1.
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
use std::sync::Arc;

use clap::{Parser, ValueEnum};
use log::{debug, info, trace};
use quack::arithmetic::{ModularArithmetic, MonicPolynomialEvaluator};
use quack::{PowerSumQuack, Quack, StrawmanAQuack, StrawmanBQuack};
use rand::Rng;
use tokio;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::sync::Mutex; // locked across calls to .await
use tokio::time::{Duration, Instant};

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
    /// Number of seconds to stream data before sending a timeout message.
    #[arg(long, short, default_value_t = 60)]
    timeout: u64,
    /// Number of bytes to send in the payload, including the sequence number.
    #[arg(long, short, default_value_t = 240)]
    bytes: usize,
    /// Frequency at which to send packets, in milliseconds.
    #[arg(long, short, default_value_t = 20)]
    frequency: u64,
    /// Style of quack to expect.
    #[arg(long, value_enum)]
    quack_style: Option<QuackStyle>,
    /// Port to listen on for quACKs.
    #[arg(long, default_value_t = 5103)]
    quack_port: u16,
    /// Address to send quACK resets too.
    #[arg(long, default_value = "10.42.0.1:1234")]
    reset_addr: SocketAddr,
    /// QuACK threshold.
    #[arg(long, default_value_t = 8)]
    threshold: usize,
}

/// NACKs just have 4 bytes for the sequence number.
const NACK_BUFFER_SIZE: usize = 4;

/// The sidecar sniffs at a certain offset in QUIC packets such that those
/// bytes are randomly-encrypted. I don't want to edit the sidecar code
/// currently so I will set the sequence numbers here in the same offset.
/// The sidecar offset also includes the Ethernet/IP/UDP headers since it
/// sniffs from a raw socket.
/// The randomly-encrypted payload in a QUIC packet with a short header is at
/// offset 63, including the Ethernet (14), IP (20), UDP (8) headers.
const ID_OFFSET: usize = 63 - (14 + 20 + 8);

/// Max UDP payload size to expect.
const MTU: usize = 1500;

/// A packet is considered missing if a packet with a sequence number greater
/// than this threshold away has been received. So packet 4 is considered
/// missing if packet 7 or greater has been received. If the last received
/// value is 7, at most packets 5 and 6 can be considered indeterminate.
const REORDER_THRESHOLD: u32 = 3;

#[derive(Clone)]
struct PacketSender {
    sidecar: bool,
    channel: mpsc::Sender<(u32, u32)>,
    seqno_ids: Arc<Mutex<Vec<(u32, u32)>>>,
}

impl PacketSender {
    async fn new(sidecar: bool, channel: mpsc::Sender<(u32, u32)>) -> io::Result<Self> {
        Ok(Self {
            sidecar,
            channel,
            seqno_ids: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Send a packet with this sequence number to the server.
    async fn send(&mut self, seqno: u32) -> io::Result<()> {
        let id: u32 = rand::thread_rng().gen();

        // Add the new packet to the buffer and send the packet.
        // (may be some harmless reordering here)
        if self.sidecar {
            self.seqno_ids.lock().await.push((seqno, id));
        }
        self.channel.send((seqno, id)).await.unwrap();
        Ok(())
    }
}

/// Listen to the mpsc channel and actually send packets on the UDP socket.
/// Receives sequence numbers and random identifiers and fills the packets.
async fn send_data(
    sock: Arc<UdpSocket>,
    bytes: usize,
    mut rx: mpsc::Receiver<(u32, u32)>,
) -> io::Result<()> {
    let mut payload = vec![0xFF; bytes];
    tokio::spawn(async move {
        while let Some((seqno, id)) = rx.recv().await {
            // Set the sequence number in the first 4 bytes.
            let seqno_bytes = seqno.to_be_bytes();
            payload[0] = seqno_bytes[0];
            payload[1] = seqno_bytes[1];
            payload[2] = seqno_bytes[2];
            payload[3] = seqno_bytes[3];

            // Set the random packet identifier at the QUIC offset.
            let id_bytes = id.to_be_bytes();
            payload[ID_OFFSET] = id_bytes[0];
            payload[ID_OFFSET + 1] = id_bytes[1];
            payload[ID_OFFSET + 2] = id_bytes[2];
            payload[ID_OFFSET + 3] = id_bytes[3];

            sock.send(&payload).await.unwrap();
        }
    });
    Ok(())
}

/// Spawn a thread that listens for end-to-end NACKs and retransmit packets
/// when requested.
fn listen_for_nacks(sock: Arc<UdpSocket>, mut sender: PacketSender) {
    let mut buf: [u8; NACK_BUFFER_SIZE] = [0; NACK_BUFFER_SIZE];
    tokio::spawn(async move {
        loop {
            let len = sock.recv(&mut buf).await.unwrap();
            assert_eq!(len, NACK_BUFFER_SIZE);
            let seqno = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
            debug!("retransmit {} from nack", seqno);
            sender.send(seqno).await.unwrap();
        }
    });
}

/// Spawn a thread that listens for sidecar quACKs using Strawman 1a (echo
/// every identifier) and retransmit packets when determined missing.
fn listen_for_quacks_strawman_a(mut _sender: PacketSender, quack_port: u16) {
    tokio::spawn(async move {
        let sock = UdpSocket::bind(format!("0.0.0.0:{}", quack_port))
            .await
            .unwrap();
        let mut buf = vec![0; MTU];
        loop {
            let (len, _) = sock.recv_from(&mut buf).await.unwrap();
            let _quack: StrawmanAQuack = bincode::deserialize(&buf[..len]).unwrap();
            unimplemented!()
        }
    });
}

/// Spawn a thread that listens for sidecar quACKs using Strawman 1b (echo a
/// sliding window of identifiers) and retransmit packets when determined
/// missing.
fn listen_for_quacks_strawman_b(mut _sender: PacketSender, quack_port: u16) {
    tokio::spawn(async move {
        let sock = UdpSocket::bind(format!("0.0.0.0:{}", quack_port))
            .await
            .unwrap();
        let mut buf = vec![0; MTU];
        loop {
            let (len, _) = sock.recv_from(&mut buf).await.unwrap();
            let _quack: StrawmanBQuack = bincode::deserialize(&buf[..len]).unwrap();
            unimplemented!()
        }
    });
}

/// Spawn a thread that listens for sidecar quACKs using Strawman 1c (echo
/// every identifier over TCP) and retransmit packets when determined missing.
fn listen_for_quacks_strawman_c(mut _sender: PacketSender, _quack_port: u16) {
    tokio::spawn(async move { unimplemented!() });
}

/// Spawn a thread that listens for sidecar quACKs using the power sum quACK
/// and retransmit packets when determined missing.
fn listen_for_quacks_power_sum(
    mut sender: PacketSender,
    quack_port: u16,
    reset_addr: SocketAddr,
    threshold: usize,
) {
    tokio::spawn(async move {
        let sock = UdpSocket::bind(format!("0.0.0.0:{}", quack_port))
            .await
            .unwrap();
        let mut buf = vec![0; MTU];
        let mut my_quack: PowerSumQuack<u32> = PowerSumQuack::new(threshold);
        info!("listening for quacks on {:?}", sock.local_addr());

        // Variables for sending quack resets.
        let mut last_quack_reset = None;
        let sidecar_reset_threshold = Duration::from_millis(100);

        loop {
            // Deserialize the quACK and only process it if at least one packet
            // has been received and the quack has changed.
            let (len, _) = sock.recv_from(&mut buf).await.unwrap();
            let quack: PowerSumQuack<u32> = bincode::deserialize(&buf[..len]).unwrap();
            trace!(
                "received quack count={} last_value={}",
                quack.count(),
                quack.last_value()
            );
            if quack.last_value() == my_quack.last_value() {
                continue;
            }

            // Update our own cumulative quACK to include up to the last value
            // received (we would have sent everything in order).
            let mut seqno_ids = sender.seqno_ids.lock().await;
            let mut last_index_inserted = None;
            for (i, &(_, id)) in seqno_ids.iter().enumerate() {
                if id == quack.last_value() {
                    last_index_inserted = Some(i);
                    break;
                }
            }
            if let Some(idx) = last_index_inserted {
                for &(seqno, id) in seqno_ids.iter().take(idx + 1) {
                    my_quack.insert(id);
                    trace!("quack insert {} ({})", id, seqno);
                }
            }

            // Reset the quack if 1) the log got messed up above, 2) we're
            // still waiting to process a previous reset, or 3) the number of
            // missing packets exceeds the threshold.
            let now = Instant::now();
            let reset0 = last_index_inserted.is_none();
            let reset1 = my_quack.count() < quack.count();
            let reset2 = my_quack.count() > quack.count() + threshold as u32;
            if reset0 || reset1 || reset2 {
                let should_reset = if let Some(last_quack_reset) = last_quack_reset {
                    now > last_quack_reset + sidecar_reset_threshold
                } else {
                    true
                };
                if should_reset {
                    info!(
                        "reset: reordered? {} retx? {} exceeds threshold? {}",
                        reset0, reset1, reset2
                    );
                    sock.send_to(&[0], reset_addr).await.unwrap();
                    my_quack = PowerSumQuack::new(threshold);
                    *seqno_ids = vec![];
                    last_quack_reset = Some(now);
                }
                continue;
            }

            let last_index_inserted = last_index_inserted.unwrap();
            if last_quack_reset.is_some() {
                info!("successful reset");
                last_quack_reset = None;
            }

            // If the number of missing packets exceeds the threshold, reset
            // the quack. If no packets are missing, continue on.
            trace!(
                "quack counts {} - {} (last values {} {})",
                my_quack.count(),
                quack.count(),
                my_quack.last_value(),
                quack.last_value()
            );
            let diff_quack = my_quack.clone() - quack;
            if diff_quack.count() == 0 {
                seqno_ids.drain(..(last_index_inserted + 1));
                continue;
            }

            // Identify the missing sequence numbers up to the last value
            // received.
            let coeffs = diff_quack.to_coeffs();
            let mut missing_seqno_ids = Vec::new();
            for &(seqno, id) in seqno_ids.iter() {
                if id == diff_quack.last_value() {
                    break;
                }
                if MonicPolynomialEvaluator::eval(&coeffs, id).is_zero() {
                    missing_seqno_ids.push((seqno, id));
                }
            }

            // Retransmit any missing packets.
            seqno_ids.drain(..(last_index_inserted + 1));
            drop(seqno_ids);
            for (seqno, id) in missing_seqno_ids.into_iter() {
                my_quack.remove(id);
                debug!("retransmit {} from quack", seqno);
                sender.send(seqno).await.unwrap();
            }
        }
    });
}

/// Send a stream of packets at the specified frequency with the given payload.
/// When the timeout is reached, send several timeout packets and return.
async fn stream_data(
    mut sender: PacketSender,
    timeout: Duration,
    frequency: Duration,
) -> io::Result<()> {
    let mut interval = tokio::time::interval(frequency);
    let start = Instant::now();

    // Send packets with increasing sequence numbers until the elapsed time
    // is greater than the timeout.
    for seqno in 1..u32::MAX {
        interval.tick().await;
        trace!("send {}", seqno);
        sender.send(seqno).await?;
        if Instant::now() - start > timeout {
            break;
        }
    }

    // Send the timeout message. Do it a bunch and hope one makes it through.
    info!("sending timeout message");
    for _ in 0..100 {
        sender.send(u32::MAX).await?;
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    env_logger::init();

    let args = Cli::parse();
    let (tx, rx) = mpsc::channel(100);

    let sock = {
        let sock = UdpSocket::bind("0.0.0.0:0").await?;
        info!("sending from {:?}", sock.local_addr().unwrap());
        sock.connect(args.server_addr).await?;
        Arc::new(sock)
    };
    let sender = PacketSender::new(args.quack_style.is_some(), tx).await?;
    send_data(sock.clone(), args.bytes, rx).await?;
    listen_for_nacks(sock, sender.clone());
    if let Some(quack_style) = args.quack_style {
        match quack_style {
            QuackStyle::StrawmanA => listen_for_quacks_strawman_a(sender.clone(), args.quack_port),
            QuackStyle::StrawmanB => listen_for_quacks_strawman_b(sender.clone(), args.quack_port),
            QuackStyle::StrawmanC => listen_for_quacks_strawman_c(sender.clone(), args.quack_port),
            QuackStyle::PowerSum => listen_for_quacks_power_sum(
                sender.clone(),
                args.quack_port,
                args.reset_addr,
                args.threshold,
            ),
        };
    }
    stream_data(
        sender,
        Duration::from_secs(args.timeout),
        Duration::from_millis(args.frequency),
    )
    .await?;
    Ok(())
}
