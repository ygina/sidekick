use clap::Parser;
use quack::PowerSumQuack;
use sidecar::sidecar_multi::start_sidecar_multi;
use sidecar::SidecarMulti;
use signal_hook::{consts::SIGTERM, iterator::Signals};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::time::{self, Duration, Instant};

#[derive(Parser)]
struct Cli {
    /// The threshold number of missing packets.
    #[arg(long)]
    threshold: usize,
    /// Frequency at which to quack, in ms. If frequency is 0, does not quack.
    #[arg(long, short = 'f')]
    frequency: u64,
    /// Interface to listen on.
    #[arg(long, short = 'i', default_value = "r1-eth1")]
    interface: String,
    /// My IPv4 address to receive quACK resets.
    #[arg(long = "my-ip", default_value = "10.0.2.1")]
    my_ip: Ipv4Addr,
    /// My port to receive quACK resets.
    #[arg(long = "my-port", default_value_t = 1234)]
    my_port: u16,
}

pub struct Benchmark {
    pub sc: Arc<Mutex<SidecarMulti>>,
    pub frequency: Option<Duration>,
    pub my_addr: [u8; 6],
}

async fn handle_signals(sc: Arc<Mutex<SidecarMulti>>, mut signals: Signals) {
    for _ in &mut signals {
        let sc = sc.lock().unwrap();
        if let Some(start_time) = sc.start_time {
            let total = Instant::now() - start_time;
            let senders = sc.senders();
            let total_count: u32 = senders.values().map(|quack| quack.count()).sum();
            let avg_count = (total_count as usize) / senders.len();
            println!("Total time: {:?}", total);
            println!("Unique connections: {}", senders.len());
            println!("Packet count (total): {}", total_count);
            println!("Packet count (average): {}", avg_count);

            let total_us: u128 = total.as_micros();
            let rate_pps: f64 = avg_count as f64 * 1000000.0 / total_us as f64;
            let rate_mbits: f64 = rate_pps * 1500.0 * 8.0 / 1000000.0;
            println!("Average rate (packets/s): {:.3}", rate_pps);
            println!("Average rate (Mbit/s): {:.3}", rate_mbits);
            println!(
                "Combined rate (packets/s): {:.3}",
                rate_pps * (senders.len() as f64)
            );
            println!(
                "Combined rate (Mbit/s): {:.3}",
                rate_mbits * (senders.len() as f64)
            );
        } else {
            println!("No start time!");
        }
        println!("DONE");
    }
}

impl Benchmark {
    pub fn new(sc: SidecarMulti, frequency_ms: u64, my_ip: Ipv4Addr, my_port: u16) -> Self {
        let frequency = if frequency_ms == 0 {
            None
        } else {
            Some(Duration::from_millis(frequency_ms))
        };
        let mut my_addr = [0; 6];
        my_addr[0] = my_ip.octets()[0];
        my_addr[1] = my_ip.octets()[1];
        my_addr[2] = my_ip.octets()[2];
        my_addr[3] = my_ip.octets()[3];
        my_addr[4] = my_port.to_be_bytes()[0];
        my_addr[5] = my_port.to_be_bytes()[1];
        Self {
            sc: Arc::new(Mutex::new(sc)),
            frequency,
            my_addr,
        }
    }

    pub fn setup_signal_handler(&self) {
        let signals = Signals::new(&[SIGTERM]).unwrap();
        tokio::spawn(handle_signals(self.sc.clone(), signals));
    }

    pub async fn start(&mut self) {
        // Wait for the first packet to arrive.
        start_sidecar_multi(self.sc.clone(), self.my_addr)
            .unwrap()
            .await
            .unwrap();
        if let Some(frequency) = self.frequency {
            let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
            let mut interval = time::interval(frequency);
            interval.tick().await; // The first tick completes immediately.
            loop {
                interval.tick().await;
                for (key, quack) in self.sc.lock().unwrap().senders() {
                    let src_ip = IpAddr::V4(Ipv4Addr::new(key[0], key[1], key[2], key[3]));
                    let src_port = u16::from_be_bytes([key[5], key[6]]);
                    let src_addr = SocketAddr::new(src_ip, src_port);
                    let bytes = bincode::serialize(&quack).unwrap();
                    socket.send_to(&bytes, src_addr).await.unwrap();
                }
            }
        } else {
            // Park this thread until the program is killed externally.
            time::sleep(Duration::from_secs(10000)).await;
            unreachable!()
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    env_logger::init();

    let args = Cli::parse();
    let sc = SidecarMulti::new(&args.interface, args.threshold, 32);

    let mut benchmark_multi = Benchmark::new(sc, args.frequency, args.my_ip, args.my_port);
    benchmark_multi.setup_signal_handler();
    benchmark_multi.start().await;
    Ok(())
}
