use clap::Parser;
use log::{info, warn};
use quack::*;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
struct Cli {
    /// Number of trials.
    #[arg(long = "trials", default_value_t = 10)]
    num_trials: usize,
    /// Number of sent packets.
    #[arg(short = 'n', default_value_t = 1000)]
    num_packets: usize,
    /// Number of connections.
    #[arg(short = 'c', long = "connections", default_value_t = 1)]
    num_conns: usize,
    /// The threshold number of dropped packets.
    #[arg(long, short = 't', default_value_t = 20)]
    threshold: usize,
}

type AddrKey = [u8; 12];

pub fn print_summary(d: Vec<Duration>, num_packets: usize) {
    let size = d.len() as u32;
    let avg = if d.is_empty() {
        Duration::new(0, 0)
    } else {
        d.into_iter().sum::<Duration>() / size
    };
    warn!("SUMMARY: num_trials = {}, avg = {:?}", size, avg);
    let d_per_packet = avg / num_packets as u32;
    let ns_per_packet = d_per_packet.as_secs() * 1000000000 + d_per_packet.subsec_nanos() as u64;
    let packets_per_s = 1000000000 / ns_per_packet;
    warn!(
        "SUMMARY (per-packet): {:?}/packet = {} packets/s",
        d_per_packet, packets_per_s
    )
}

pub fn gen_numbers<T>(num_packets: usize) -> Vec<T>
where
    Standard: Distribution<T>,
{
    (0..num_packets).map(|_| rand::thread_rng().gen()).collect()
}

fn benchmark(threshold: usize, num_packets: usize, num_conns: usize) -> Duration {
    let numbers = gen_numbers(num_packets);
    let conns = gen_numbers::<AddrKey>(num_conns);
    let conn_numbers = gen_numbers::<usize>(num_packets)
        .into_iter()
        .enumerate()
        .map(|(i, index)| (conns[index % num_conns], numbers[i]))
        .collect::<Vec<(AddrKey, u32)>>();

    // Construct an empty data structure for the quacks.
    let mut senders: HashMap<AddrKey, PowerSumQuackU32> = HashMap::new();

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for (conn, number) in conn_numbers.into_iter() {
        senders
            .entry(conn)
            .or_insert(PowerSumQuackU32::new(threshold))
            .insert(number);
    }
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!(
        "Insert {} numbers into {} PowerSumQuackU32 (threshold = {}): {:?}",
        num_packets, num_conns, threshold, duration
    );
    duration
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();

    quack::global_config_set_max_power_sum_threshold(args.threshold);

    let mut durations: Vec<Duration> = vec![];
    for i in 0..(args.num_trials + 1) {
        let duration = benchmark(args.threshold, args.num_packets, args.num_conns);
        if i != 0 {
            durations.push(duration);
        }
    }
    print_summary(durations, args.num_packets);
}
