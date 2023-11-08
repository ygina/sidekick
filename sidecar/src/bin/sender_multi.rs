use clap::Parser;
use log::{debug, info};
use quack::PowerSumQuack;
use sidecar::{
    sidecar_multi::{start_sidecar_multi, start_sidecar_multi_frequency_pkts},
    SidecarMulti,
};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::sync::oneshot;
use tokio::time::{self, Duration, Instant};

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
    #[arg(long = "frequency-ms")]
    frequency_ms: Option<u64>,
    /// Frequency at which to quack, in packets.
    #[arg(long = "frequency-pkts")]
    frequency_pkts: Option<u32>,
    /// Address of the UDP socket to quack to e.g., <IP:PORT>.
    #[arg(long = "quack-addr", default_value = "10.42.0.250:5104")]
    quack_addr: SocketAddr,
    /// My IPv4 address to receive quACK resets.
    #[arg(long = "my-ip", default_value = "10.42.0.1")]
    my_ip: Ipv4Addr,
    /// My port to receive quACK resets.
    #[arg(long = "my-port", default_value_t = 1234)]
    my_port: u16,
    /// Destination IP.
    #[arg(long = "dst-ip", default_value = "34.221.237.169")]
    dst_ip: Ipv4Addr,
    /// Destination port.
    #[arg(long = "dst-port", default_value_t = 443)]
    dst_port: u16,
}

async fn send_quacks_ms(
    sc: Arc<Mutex<SidecarMulti>>,
    rx: oneshot::Receiver<Instant>,
    dst_key: [u8; 6],
    quack_addr: SocketAddr,
    frequency_ms: u64,
) {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .expect(&format!("error binding to UDP socket"));
    let mut interval = time::interval(Duration::from_millis(frequency_ms));
    // The first tick completes immediately
    interval.tick().await;
    rx.await
        .expect("couldn't receive notice that 1st packet was sniffed");
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
    info!(
        "interface={} threshold={} bits={} frequency_ms={:?} frequency_pkts={:?}",
        args.interface, args.threshold, args.num_bits_id, args.frequency_ms, args.frequency_pkts
    );

    // Start the sidecar.
    let sc = SidecarMulti::new(&args.interface, args.threshold, args.num_bits_id);

    // Get the target dst key. If the dst of the traffic matches this key,
    // send a quack.
    let mut dst_key: [u8; 6] = [0; 6];
    dst_key[0] = args.dst_ip.octets()[0];
    dst_key[1] = args.dst_ip.octets()[1];
    dst_key[2] = args.dst_ip.octets()[2];
    dst_key[3] = args.dst_ip.octets()[3];
    dst_key[4] = args.dst_port.to_be_bytes()[0];
    dst_key[5] = args.dst_port.to_be_bytes()[1];

    let mut my_addr: [u8; 6] = [0; 6];
    my_addr[0] = args.my_ip.octets()[0];
    my_addr[1] = args.my_ip.octets()[1];
    my_addr[2] = args.my_ip.octets()[2];
    my_addr[3] = args.my_ip.octets()[3];
    my_addr[4] = args.my_port.to_be_bytes()[0];
    my_addr[5] = args.my_port.to_be_bytes()[1];

    // Handle snapshotted quACKs at the specified frequency.
    info!("my address is {:?}", my_addr);
    let sc = Arc::new(Mutex::new(sc));
    if let Some(frequency_ms) = args.frequency_ms {
        assert!(frequency_ms > 0);
        let rx = start_sidecar_multi(sc.clone(), my_addr)?;
        send_quacks_ms(sc, rx, dst_key, args.quack_addr, frequency_ms).await;
    } else if let Some(frequency_pkts) = args.frequency_pkts {
        assert!(frequency_pkts > 0);
        start_sidecar_multi_frequency_pkts(sc.clone(), my_addr, frequency_pkts, args.quack_addr)
            .await
            .unwrap();
    }
    Ok(())
}
