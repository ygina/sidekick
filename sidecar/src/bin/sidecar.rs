use std::net::{UdpSocket, SocketAddr};
use std::time::Duration;
use clap::{Parser, Subcommand};
use quack::Quack;
use sidecar::{Sidecar, SidecarType};

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

/// If a target socket address and source UDP socket are provided, generates a
/// quACK handler that sends the byte-serialized quACK in a UDP packet to the
/// target address. Otherwise, prints the number of received packets to stdout.
fn gen_quack_handler(
    to_from: Option<(SocketAddr, UdpSocket)>,
) -> Box<dyn Fn(Quack)> {
    if let Some((addr, socket)) = to_from {
        Box::new(move |quack: Quack| {
            let bytes = bincode::serialize(&quack).unwrap();
            socket.send_to(&bytes, addr).unwrap();
        })
    } else {
        Box::new(move |quack: Quack| println!("quack {}", quack.count))
    }
}


fn main() {
    let args = Cli::parse();
    match args.ty {
        CliSidecarType::QuackSender {
            frequency_ms,
            frequency_packets,
            target_addr,
        } => {
            // Create the quACK handler.
            let to_from = target_addr.map(|addr| {
                let socket = UdpSocket::bind("127.0.0.1:53534").expect(
                    &format!("error binding to UDP socket: {:?}", addr));
                (addr, socket)
            });
            let handler = gen_quack_handler(to_from);

            // Start the sidecar.
            let sc = Sidecar::new(
                SidecarType::QuackSender,
                &args.interface,
                args.threshold,
                args.num_bits_id,
            );
            // TODO: async code
            sc.start();

            // Handle a snapshotted quACK at the specified frequency.
            if let Some(ms) = frequency_ms {
                loop {
                    // TODO: tokio
                    std::thread::sleep(Duration::from_millis(ms));
                    let quack = sc.quack();
                    handler(quack);
                }
            }
            if let Some(_freq) = frequency_packets {
                unimplemented!();
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
            let rx = sc.listen(port);
            loop {
                let quack = rx.recv().expect("channel has hung up");
                // TODO: tracing library
                let result = sc.quack_decode(quack);
                println!("{}", result);
            }
        }
    }
}
