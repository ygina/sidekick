use std::sync::{Arc, Mutex};
use clap::Parser;
use quack::Quack;
use sidecar::{Sidecar, SidecarType};
use log::debug;

/// Receives quACKs in the sidecar protocol, sends data in the base protocol.
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
    /// Port on which to receive quACKs. Logs quACKS to stdout. If you want
    /// to use a receiver in an actual sidecar protocol, write a binary that
    /// calls listen() in the sidecar library.
    #[arg(long = "listen-on", default_value_t = 53535)]
    port: u16,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    env_logger::init();

    let args = Cli::parse();
    debug!("interface={} threshold={} bits={} port={}",
        args.interface, args.threshold, args.num_bits_id, args.port);
    let sc = Arc::new(Mutex::new(Sidecar::new(
        SidecarType::QuackReceiver,
        &args.interface,
        args.threshold,
        args.num_bits_id,
    )));
    Sidecar::start(sc.clone())?;
    let mut rx = sc.lock().unwrap().listen(args.port);
    loop {
        let quack = rx.recv().await.expect("channel has hung up");
        let (my_quack, my_log) = sc.lock().unwrap().quack_with_log();
        let difference_quack = my_quack - quack;
        let result = difference_quack.decode_with_log(&my_log);
        debug!("{:?}", result);
    }
}
