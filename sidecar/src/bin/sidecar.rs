use std::net::SocketAddr;
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
enum CliSidecarType {
    /// Sends quACKs in the sidecar protocol, receives data in the base
    /// protocol.
    QuackSender {
        /// Frequency at which to quack, in ms. If neither frequency argument
        /// is provided, does not quack.
        #[arg(long = "frequency-ms")]
        frequency_ms: Option<usize>,
        /// Frequency at which to quack, based on the number of received
        /// packets. If neither frequency argument is provided, does not quack.
        /// If `frequency-ms' is also provided, ignores this argument.
        #[arg(long = "frequency-packets")]
        frequency_packets: Option<usize>,
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


fn main() {
    let args = Cli::parse();
    match args.ty {
        CliSidecarType::QuackSender {
            frequency_ms,
            frequency_packets,
            target_addr,
        } => {
            if let Some(_freq) = frequency_ms {
                unimplemented!();
            }
            if let Some(_freq) = frequency_packets {
                unimplemented!();
            }
            unimplemented!()
        }
        CliSidecarType::QuackReceiver { port } => {
            unimplemented!()
        }
    }
}
