use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use clap::Parser;
use sidecar::{SidecarMulti, sidecar_multi::start_sidecar_multi};
use tokio::net::UdpSocket;
use tokio::time::{self, Duration};
use log::{debug, info};
use quack::Quack;

/// Sends quACKs in the sidecar protocol, receives data in the base protocol.
#[derive(Parser)]
struct Cli {
    /// Interface to listen on e.g., `eth1'.
    #[arg(long, short = 'i', default_value = "wlp1s0")]
    interface: String,
    /// The threshold number of missing packets.
    #[arg(long, short = 't', default_value_t = 80)]
    threshold: usize,
    /// Number of identifier bits.
    #[arg(long = "bits", short = 'b', default_value_t = 32)]
    num_bits_id: usize,
    /// Frequency at which to quack, in ms.
    #[arg(long = "frequency", default_value_t = 10)]
    frequency_ms: u64,
    /// Address of the UDP socket to quack to e.g., <IP:PORT>.
    #[arg(long = "quack-addr", default_value = "10.42.0.250:5104")]
    quack_addr: SocketAddr,
    /// My IPv4 address to receive quACK resets.
    #[arg(long = "my-addr", default_value = "10.42.0.1")]
    my_addr: Ipv4Addr,
    /// Destination IP.
    #[arg(long = "dst-ip", default_value = "34.221.237.169")]
    dst_ip: Ipv4Addr,
    /// Destination port.
    #[arg(long = "dst-port", default_value_t = 443)]
    dst_port: u16,
}

async fn send_quacks(
    sc: Arc<Mutex<SidecarMulti>>,
    dst_key: [u8; 6],
    quack_addr: SocketAddr,
    frequency_ms: u64,
) {
    let socket = UdpSocket::bind("0.0.0.0:0").await.expect(
        &format!("error binding to UDP socket"));
    let mut interval = time::interval(Duration::from_millis(frequency_ms));
    // The first tick completes immediately
    interval.tick().await;
    loop {
        interval.tick().await;
        let sc = sc.lock().unwrap();
        for (key, quack) in sc.senders().iter() {
            if &key[6..] == &dst_key {
                let bytes = bincode::serialize(&quack).unwrap();
                debug!("quack {} key {:?}", quack.count(), key);
                socket.send_to(&bytes, quack_addr).await.unwrap();
                break;
            }
        }
        drop(sc);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    env_logger::init();

    let args = Cli::parse();
    assert!(args.frequency_ms > 0);
    info!("interface={} threshold={} bits={} frequency_ms={}",
        args.interface, args.threshold, args.num_bits_id, args.frequency_ms);

    // Start the sidecar.
    let sc = SidecarMulti::new(
        &args.interface,
        args.threshold,
        args.num_bits_id,
    );

    // Get the target dst key. If the dst of the traffic matches this key,
    // send a quack.
    let mut dst_key: [u8; 6] = [0; 6];
    dst_key[0] = args.dst_ip.octets()[0];
    dst_key[1] = args.dst_ip.octets()[1];
    dst_key[2] = args.dst_ip.octets()[2];
    dst_key[3] = args.dst_ip.octets()[3];
    dst_key[4] = args.dst_port.to_be_bytes()[0];
    dst_key[5] = args.dst_port.to_be_bytes()[1];

    // Handle snapshotted quACKs at the specified frequency.
    info!("my ipv4 address is {:?}", args.my_addr);
    let sc = Arc::new(Mutex::new(sc));
    start_sidecar_multi(sc.clone(), args.my_addr.octets())?;
    send_quacks(sc, dst_key, args.quack_addr, args.frequency_ms).await;
    Ok(())
}
