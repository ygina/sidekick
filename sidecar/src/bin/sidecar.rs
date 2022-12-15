use std::net::SocketAddr;
use clap::{Parser, Subcommand};
use sidecar::{Sidecar, SidecarType};
use tokio::net::UdpSocket;
use tokio::time::{self, Duration};
use tokio::sync::oneshot;
use log::{debug, info};

#[derive(Subcommand, Debug)]
enum CliSidecarType {
    /// Sends quACKs in the sidecar protocol, receives data in the base
    /// protocol.
    QuackSender {
        /// Frequency at which to quack, in ms. If frequency is 0, does not
        /// quack.
        #[arg(long = "frequency-ms", default_value_t = 1000)]
        frequency_ms: u64,
        /// Address of the UDP socket to quack to e.g., <IP:PORT>. If missing,
        /// goes to stdout.
        #[arg(long = "target-addr")]
        target_addr: Option<SocketAddr>,
    },
    /// Receives quACKs in the sidecar protocol, sends data in the base
    /// protocol.
    QuackReceiver {
        /// Port on which to receive quACKs. Logs quACKS to stdout. If you want
        /// to use a receiver in an actual sidecar protocol, write a binary that
        /// calls listen() in the sidecar library.
        #[arg(long = "listen-on", default_value_t = 53535)]
        port: u16,
    },
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    ty: CliSidecarType,
    /// Interface to listen on e.g., `eth1'.
    #[arg(long)]
    interface: String,
    /// The threshold number of missing packets.
    #[arg(long, short = 't', default_value_t = 20)]
    threshold: usize,
    /// Number of identifier bits.
    #[arg(long = "bits", short = 'b', default_value_t = 32)]
    num_bits_id: usize,
}

async fn send_quacks(
    sc: Sidecar,
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
            let quack = sc.quack();
            let bytes = bincode::serialize(&quack).unwrap();
            println!("quack {}", quack.count);
            socket.send_to(&bytes, addr).await.unwrap();
        }
    }
}

async fn print_quacks(
    sc: Sidecar,
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
            let quack = sc.quack();
            println!("quack {}", quack.count);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::init();

    let args = Cli::parse();
    debug!("sidecar_type={:?}", args.ty);
    debug!("interface={} threshold={} bits={}",
        args.interface, args.threshold, args.num_bits_id);
    match args.ty {
        CliSidecarType::QuackSender {
            frequency_ms,
            target_addr,
        } => {
            debug!("frequency_ms={:?}", frequency_ms);
            // Start the sidecar.
            let sc = Sidecar::new(
                SidecarType::QuackSender,
                &args.interface,
                args.threshold,
                args.num_bits_id,
            );
            let rx = sc.start()?;

            // Handle a snapshotted quACK at the specified frequency.
            if let Some(addr) = target_addr {
                info!("quACKing to {:?}", addr);
                send_quacks(sc, rx, addr, frequency_ms).await;
            } else {
                info!("printing quACKs");
                print_quacks(sc, rx, frequency_ms).await;
            }
        }
        CliSidecarType::QuackReceiver { port } => {
            let sc = Sidecar::new(
                SidecarType::QuackReceiver,
                &args.interface,
                args.threshold,
                args.num_bits_id,
            );
            sc.start()?;
            let mut rx = sc.listen(port);
            loop {
                let quack = rx.recv().await.expect("channel has hung up");
                let result = sc.quack_decode(quack);
                debug!("{}", result);
            }
        }
    }
    Ok(())
}
