use std::net::SocketAddr;
use std::time::Duration;
use clap::{Parser, Subcommand};
use sidecar::{Sidecar, SidecarType};
use tokio::net::UdpSocket;

#[derive(Subcommand)]
enum CliSidecarType {
    /// Sends quACKs in the sidecar protocol, receives data in the base
    /// protocol.
    QuackSender {
        /// Frequency at which to quack, in ms. If neither frequency argument
        /// is provided, does not quack.
        #[arg(long = "frequency-ms")]
        frequency_ms: Option<u64>,
        /// Frequency at which to quack, based on the number of received
        /// packets. If neither frequency argument is provided, does not quack.
        /// If `frequency-ms' is also provided, ignores this argument.
        #[arg(long = "frequency-packets")]
        frequency_packets: Option<usize>,
        /// Address of the UDP socket to quack to e.g., <IP:PORT>. If missing,
        /// goes to stdout. Sends from 127.0.0.1:53534.
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
    addr: SocketAddr,
    frequency_ms: Option<u64>,
    frequency_packets: Option<usize>,
) {
    let socket = UdpSocket::bind("127.0.0.1:53534").await.expect(
        &format!("error binding to UDP socket: 127.0.0.1:53534"));
    if let Some(ms) = frequency_ms {
        loop {
            tokio::time::sleep(Duration::from_millis(ms)).await;
            let quack = sc.quack();
            let bytes = bincode::serialize(&quack).unwrap();
            println!("quack {}", quack.count);
            socket.send_to(&bytes, addr).await.unwrap();
        }
    }
    if let Some(_) = frequency_packets {
        unimplemented!()
    }
}

async fn print_quacks(
    sc: Sidecar,
    frequency_ms: Option<u64>,
    frequency_packets: Option<usize>,
) {
    if let Some(ms) = frequency_ms {
        loop {
            tokio::time::sleep(Duration::from_millis(ms)).await;
            let quack = sc.quack();
            println!("quack {}", quack.count);
        }
    }
    if let Some(_) = frequency_packets {
        unimplemented!()
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Cli::parse();
    let rt = tokio::runtime::Runtime::new().unwrap();
    match args.ty {
        CliSidecarType::QuackSender {
            frequency_ms,
            frequency_packets,
            target_addr,
        } => {
            // Start the sidecar.
            let sc = Sidecar::new(
                SidecarType::QuackSender,
                &args.interface,
                args.threshold,
                args.num_bits_id,
            );
            // TODO: async code
            sc.start(&rt)?;

            // Handle a snapshotted quACK at the specified frequency.
            if let Some(addr) = target_addr {
                send_quacks(sc, addr, frequency_ms, frequency_packets).await;
            } else {
                print_quacks(sc, frequency_ms, frequency_packets).await;
            }
        }
        CliSidecarType::QuackReceiver { port } => {
            let sc = Sidecar::new(
                SidecarType::QuackReceiver,
                &args.interface,
                args.threshold,
                args.num_bits_id,
            );
            // TODO: async code
            let mut rx = sc.listen(port, &rt);
            loop {
                let quack = rx.recv().await.unwrap();
                // TODO: tracing library
                let result = sc.quack_decode(quack);
                println!("result {}", result);
            }
        }
    }
    Ok(())
}
