use clap::Parser;
use log::{debug, info, trace};
use quack::PowerSumQuack;
use sidekick::Sidekick;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::sync::oneshot;
use tokio::time::{self, Duration};

/// Sends quACKs in the sidekick protocol, receives data in the base protocol.
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
    /// My IPv4 address to receive quACK resets.
    #[arg(long = "my-addr")]
    my_addr: Ipv4Addr,
}

const QUACK_SEND_PORT: u16 = 5104;

async fn send_quacks(
    sc: Arc<Mutex<Sidekick>>,
    rx: oneshot::Receiver<()>,
    addr: SocketAddr,
    frequency_ms: u64,
) {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", QUACK_SEND_PORT))
        .await
        .expect("error binding to UDP socket");
    if frequency_ms > 0 {
        rx.await
            .expect("couldn't receive notice that 1st packet was sniffed");
        let mut interval = time::interval(Duration::from_millis(frequency_ms));
        // The first tick completes immediately
        interval.tick().await;
        loop {
            interval.tick().await;
            let quack = sc.lock().unwrap().quack();
            let bytes = bincode::serialize(&quack).unwrap();
            trace!("quack {}", quack.count());
            if socket.send_to(&bytes, addr).await.is_err() {
                break;
            }
        }
    }
}

async fn print_quacks(sc: Arc<Mutex<Sidekick>>, rx: oneshot::Receiver<()>, frequency_ms: u64) {
    if frequency_ms > 0 {
        rx.await
            .expect("couldn't receive notice that 1st packet was sniffed");
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
    debug!(
        "interface={} threshold={} bits={}",
        args.interface, args.threshold, args.num_bits_id
    );
    debug!(
        "frequency_ms={:?} frequency_pkts={:?} target_addr={:?}",
        args.frequency_ms, args.frequency_pkts, args.target_addr
    );

    // Start the sidekick.
    let mut sc = Sidekick::new(&args.interface, args.threshold, args.num_bits_id);

    // Handle a snapshotted quACK at the specified frequency.
    if let Some(frequency_ms) = args.frequency_ms {
        info!("my ipv4 address is {:?}", args.my_addr);
        let sc = Arc::new(Mutex::new(sc));
        let rx = Sidekick::start(sc.clone(), args.my_addr.octets(), QUACK_SEND_PORT)?;
        if let Some(addr) = args.target_addr {
            info!("quACKing to {:?}", addr);
            send_quacks(sc, rx, addr, frequency_ms).await;
        } else {
            info!("printing quACKs");
            print_quacks(sc, rx, frequency_ms).await;
        }
    } else if let Some(frequency_pkts) = args.frequency_pkts {
        let addr = args.target_addr.expect("Address must be set");
        sc.start_frequency_pkts(args.my_addr.octets(), QUACK_SEND_PORT, frequency_pkts, addr)
            .await
            .unwrap();
    }
    Ok(())
}
