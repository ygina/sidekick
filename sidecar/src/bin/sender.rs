use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use clap::Parser;
use sidecar::Sidecar;
use tokio::net::UdpSocket;
use tokio::time::{self, Duration};
use tokio::sync::oneshot;
use log::{debug, info};
use quack::Quack;

/// Sends quACKs in the sidecar protocol, receives data in the base protocol.
#[derive(Parser)]
struct Cli {
    /// Interface to listen on e.g., `eth1'.
    #[arg(long, short = 'i')]
    interface: String,
    /// The threshold number of missing packets.
    #[arg(long, short = 't', default_value_t = 20)]
    threshold: usize,
    /// Number of identifier bits.
    #[arg(long = "bits", short = 'b', default_value_t = 32)]
    num_bits_id: usize,
    /// Frequency at which to quack, in ms. If frequency is 0, does not quack.
    #[arg(long = "frequency-ms")]
    frequency_ms: Option<u64>,
    /// Frequency at which to quack, in packets.
    #[arg(long = "frequency-pkts")]
    frequency_pkts: Option<usize>,
    /// Address of the UDP socket to quack to e.g., <IP:PORT>. If missing,
    /// goes to stdout.
    #[arg(long = "target-addr")]
    target_addr: Option<SocketAddr>,
}

async fn send_quacks(
    sc: Arc<Mutex<Sidecar>>,
    rx: oneshot::Receiver<()>,
    addr: SocketAddr,
    frequency_ms: u64,
) {
    let socket = UdpSocket::bind("0.0.0.0:0").await.expect(
        &format!("error binding to UDP socket"));
    if frequency_ms > 0 {
        rx.await.expect("couldn't receive notice that 1st packet was sniffed");
        let mut interval = time::interval(Duration::from_millis(frequency_ms));
        // The first tick completes immediately
        interval.tick().await;
        loop {
            interval.tick().await;
            let quack = sc.lock().unwrap().quack();
            let bytes = bincode::serialize(&quack).unwrap();
            info!("quack {}", quack.count());
            socket.send_to(&bytes, addr).await.unwrap();
        }
    }
}

async fn print_quacks(
    sc: Arc<Mutex<Sidecar>>,
    rx: oneshot::Receiver<()>,
    frequency_ms: u64,
) {
    if frequency_ms > 0 {
        rx.await.expect("couldn't receive notice that 1st packet was sniffed");
        let mut interval = time::interval(Duration::from_millis(frequency_ms));
        // The first tick completes immediately
        interval.tick().await;
        loop {
            interval.tick().await;
            let quack = sc.lock().unwrap().quack();
            info!("quack {}", quack.count());
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    env_logger::init();

    let args = Cli::parse();
    debug!("interface={} threshold={} bits={}",
        args.interface, args.threshold, args.num_bits_id);
    debug!("frequency_ms={:?} frequency_pkts={:?} target_addr={:?}",
        args.frequency_ms, args.frequency_pkts, args.target_addr);

    // Start the sidecar.
    let mut sc = Sidecar::new(
        &args.interface,
        args.threshold,
        args.num_bits_id,
    );

    // Handle a snapshotted quACK at the specified frequency.
    if let Some(frequency_ms) = args.frequency_ms {
        let sc = Arc::new(Mutex::new(sc));
        let rx = Sidecar::start(sc.clone())?;
        if let Some(addr) = args.target_addr {
            info!("quACKing to {:?}", addr);
            send_quacks(sc, rx, addr, frequency_ms).await;
        } else {
            info!("printing quACKs");
            print_quacks(sc, rx, frequency_ms).await;
        }
    } else if let Some(frequency_pkts) = args.frequency_pkts {
        let addr = args.target_addr.expect("Address must be set");
        sc.start_frequency_pkts(frequency_pkts, addr).await.unwrap();
    }
    Ok(())
}
