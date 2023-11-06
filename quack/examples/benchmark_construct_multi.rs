use clap::{Parser, ValueEnum};
use log::{debug, info, warn};
use quack::{
    arithmetic::{ModularArithmetic, ModularInteger},
    *,
};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::ops::{AddAssign, MulAssign, Sub, SubAssign};
use std::time::{Duration, Instant};

#[derive(Clone, ValueEnum, Debug, PartialEq, Eq)]
pub enum QuackType {
    Strawman1a,
    Strawman1b,
    Strawman2,
    PowerSum,
}

#[derive(Parser, Debug)]
pub struct QuackParams {
    /// The threshold number of dropped packets.
    #[arg(long, short = 't', default_value_t = 20)]
    threshold: usize,
    /// Number of identifier bits.
    #[arg(long = "bits", short = 'b', default_value_t = 32)]
    num_bits_id: usize,
    /// Enable pre-computation optimization
    #[arg(long)]
    precompute: bool,
    /// Disable not-factoring optimization
    #[arg(long)]
    factor: bool,
    /// Enable Montgomery multiplication optimization
    #[arg(long)]
    montgomery: bool,
}

#[derive(Parser, Debug)]
struct Cli {
    /// Quack type.
    #[arg(value_enum)]
    quack_ty: QuackType,
    /// Number of trials.
    #[arg(long = "trials", default_value_t = 10)]
    num_trials: usize,
    /// Number of sent packets.
    #[arg(short = 'n', default_value_t = 1000)]
    num_packets: usize,
    /// Number of dropped packets.
    #[arg(short = 'd', long = "dropped", default_value_t = 20)]
    num_drop: usize,
    /// Number of connections.
    #[arg(short = 'c', long = "connections", default_value_t = 1)]
    num_conns: usize,
    /// Quack parameters.
    #[command(flatten)]
    quack: QuackParams,
}

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

type AddrKey = [u8; 12];

fn benchmark_construct_power_sum<T>(
    size: usize,
    num_bits_id: usize,
    num_packets: usize,
    num_conns: usize,
) -> Duration
where
    Standard: Distribution<T>,
    T: Debug + Display + Default + PartialOrd + Sub<Output = T> + Copy,
    ModularInteger<T>: ModularArithmetic<T> + AddAssign + MulAssign + SubAssign,
{
    let numbers = gen_numbers::<T>(num_packets);
    let conns = gen_numbers::<AddrKey>(num_conns);
    let conn_numbers = gen_numbers::<usize>(num_packets)
        .into_iter()
        .enumerate()
        .map(|(i, index)| (conns[index % num_conns], numbers[i]))
        .collect::<Vec<(AddrKey, T)>>();

    // Construct an empty data structure for the quacks.
    let mut senders: HashMap<AddrKey, PowerSumQuack<T>> = HashMap::new();

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for (conn, number) in conn_numbers.into_iter() {
        senders
            .entry(conn)
            .or_insert(PowerSumQuack::new(size))
            .insert(number);
    }
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!(
        "Insert {} numbers into {} Quacks (bits = {}, threshold = {}): {:?}",
        num_packets, num_conns, num_bits_id, size, duration
    );
    duration
}

pub fn run_benchmark(
    quack_ty: QuackType,
    num_trials: usize,
    num_packets: usize,
    num_conns: usize,
    params: QuackParams,
) {
    // Allocate buffer for benchmark durations.
    let mut durations: Vec<Duration> = vec![];

    for i in 0..(num_trials + 1) {
        assert_eq!(quack_ty, QuackType::PowerSum);
        assert!(!params.precompute);
        let duration = match params.num_bits_id {
            16 => benchmark_construct_power_sum::<u16>(
                params.threshold,
                params.num_bits_id,
                num_packets,
                num_conns,
            ),
            32 => benchmark_construct_power_sum::<u32>(
                params.threshold,
                params.num_bits_id,
                num_packets,
                num_conns,
            ),
            64 => benchmark_construct_power_sum::<u64>(
                params.threshold,
                params.num_bits_id,
                num_packets,
                num_conns,
            ),
            _ => unimplemented!(),
        };
        if i > 0 {
            durations.push(duration);
        }
    }
    print_summary(durations, num_packets);
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    debug!("args = {:?}", args);
    run_benchmark(
        args.quack_ty,
        args.num_trials,
        args.num_packets,
        args.num_conns,
        args.quack,
    );
}
