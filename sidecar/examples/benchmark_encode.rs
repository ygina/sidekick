use clap::Parser;
use quack::PowerSumQuack;
use sidecar::Sidecar;
use signal_hook::{consts::SIGTERM, iterator::Signals};
use std::net::{Ipv4Addr, SocketAddr};
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
    /// Address of the UDP socket to quack to e.g., <IP:PORT>.
    #[arg(long, default_value = "10.0.2.10:5103")]
    addr: SocketAddr,
    /// My IPv4 address to receive quACK resets.
    #[arg(long = "my-ip", default_value = "10.0.2.1")]
    my_ip: Ipv4Addr,
}

pub struct Benchmark {
    pub sc: Arc<Mutex<Sidecar>>,
    pub addr: SocketAddr,
    pub frequency: Option<Duration>,
}

async fn handle_signals(sc: Arc<Mutex<Sidecar>>, mut signals: Signals) {
    for _ in &mut signals {
        let sc = sc.lock().unwrap();
        if let Some(start_time) = sc.start_time {
            let total = Instant::now() - start_time;
            let count = sc.quack().count();
            println!("Total: {:?}", total);
            println!("Count: {}", count);

            let total_us: u128 = total.as_micros();
            let rate_pps: f64 = count as f64 * 1000000.0 / total_us as f64;
            let rate_mbits: f64 = rate_pps * 1500.0 * 8.0 / 1000000.0;
            println!("Rate (packets/s): {:.3}", rate_pps);
        } else {
            println!("No start time!");
        }
        println!("DONE");
    }
}

impl Benchmark {
    pub fn new(sc: Sidecar, addr: SocketAddr, frequency_ms: u64) -> Self {
        let frequency = if frequency_ms == 0 {
            None
        } else {
            Some(Duration::from_millis(frequency_ms))
        };
        Self {
            sc: Arc::new(Mutex::new(sc)),
            addr,
            frequency,
        }
    }

    pub fn setup_signal_handler(&self) {
        let signals = Signals::new(&[SIGTERM]).unwrap();
        tokio::spawn(handle_signals(self.sc.clone(), signals));
    }

    pub async fn start(&mut self, my_ipv4_addr: [u8; 4]) {
        // Wait for the first packet to arrive.
        Sidecar::start(self.sc.clone(), my_ipv4_addr)
            .unwrap()
            .await
            .unwrap();
        if let Some(frequency) = self.frequency {
            let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
            let mut interval = time::interval(frequency);
            interval.tick().await; // The first tick completes immediately.
            loop {
                interval.tick().await;
                let quack = self.sc.lock().unwrap().quack();
                let bytes = bincode::serialize(&quack).unwrap();
                socket.send_to(&bytes, self.addr).await.unwrap();
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
    let sc = Sidecar::new(&args.interface, args.threshold, 32);
    let mut benchmark = Benchmark::new(sc, args.addr, args.frequency);
    benchmark.setup_signal_handler();
    benchmark
        .start([
            args.my_ip.octets()[0],
            args.my_ip.octets()[1],
            args.my_ip.octets()[2],
            args.my_ip.octets()[3],
        ])
        .await;
    Ok(())
}
