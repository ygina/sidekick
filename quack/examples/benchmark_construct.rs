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
use serde::Serialize;
use sha2::{Digest, Sha256};
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

fn benchmark_construct_strawman1a(num_packets: usize) -> Duration {
    let numbers = gen_numbers::<u32>(num_packets);

    let mut quack = StrawmanAQuack { sidecar_id: 0 };

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for number in numbers {
        quack.sidecar_id = number;
        let _bytes = bincode::serialize(&quack).unwrap();
    }
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!(
        "Serialize {} numbers into StrawmanAQuack: {:?}",
        num_packets, duration
    );
    duration
}

fn benchmark_construct_strawman1b(threshold: usize, num_packets: usize) -> Duration {
    let numbers = gen_numbers::<u32>(num_packets);

    let mut quack = StrawmanBQuack::new(threshold);

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for number in numbers {
        quack.insert(number);
        let _bytes = bincode::serialize(&quack).unwrap();
    }
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!(
        "Serialize {} numbers into StrawmanBQuack with threshold {}: {:?}",
        num_packets, threshold, duration
    );
    duration
}

fn benchmark_construct_strawman2(num_packets: usize) -> Duration {
    let numbers = gen_numbers::<u32>(num_packets);
    let mut acc = Sha256::new();

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for number in numbers.iter().take(num_packets) {
        acc.update(number.to_be_bytes());
    }
    let _array = acc.finalize();
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!(
        "Insert {} numbers into a sha256 digest: {:?}",
        num_packets, duration
    );
    duration
}

fn benchmark_construct_power_sum_precompute_u16(threshold: usize, num_packets: usize) -> Duration {
    let numbers = gen_numbers::<u16>(num_packets);

    // Construct two empty Quacks.
    let mut quack = PowerTableQuack::new(threshold);

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for number in numbers {
        quack.insert(number);
    }
    let _bytes = bincode::serialize(&quack);
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!(
        "Insert {} numbers into 2 Quacks (bits = 16, \
        threshold = {}): {:?}",
        num_packets, threshold, duration
    );
    duration
}

fn benchmark_construct_power_sum_montgomery_u64(threshold: usize, num_packets: usize) -> Duration {
    let numbers = gen_numbers::<u64>(num_packets);

    // Construct an empty Quack.
    let mut quack = MontgomeryQuack::new(threshold);

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for number in numbers {
        quack.insert(number);
    }
    let _bytes = bincode::serialize(&quack);
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!(
        "Insert {} numbers into a montgomery quACK (bits = 64, \
        threshold = {}): {:?}",
        num_packets, threshold, duration
    );
    duration
}

fn benchmark_construct_power_sum<T>(
    threshold: usize,
    num_bits_id: usize,
    num_packets: usize,
) -> Duration
where
    Standard: Distribution<T>,
    T: Debug + Display + Default + PartialOrd + Sub<Output = T> + Copy + Serialize,
    ModularInteger<T>: ModularArithmetic<T> + AddAssign + MulAssign + SubAssign,
{
    let numbers = gen_numbers::<T>(num_packets);

    // Construct two empty Quacks.
    let mut quack = PowerSumQuack::<T>::new(threshold);

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    #[cfg(feature = "cycles")]
    let start = unsafe { core::arch::x86_64::_rdtsc() };
    for number in numbers {
        quack.insert(number);
    }
    #[cfg(feature = "cycles")]
    let end = unsafe { core::arch::x86_64::_rdtsc() };
    let _bytes = bincode::serialize(&quack);
    let t2 = Instant::now();

    let duration = t2 - t1;
    #[cfg(not(feature = "cycles"))]
    info!(
        "Insert {} numbers into a power sum quACK (bits = {}, \
        threshold = {}): {:?}",
        num_packets, num_bits_id, threshold, duration
    );
    #[cfg(feature = "cycles")]
    info!(
        "Insert {} numbers into a power sum quACK (bits = {}, \
        threshold = {}): {:?} ({} cycles/pkt)",
        num_packets,
        num_bits_id,
        threshold,
        duration,
        (end - start) / (num_packets as u64)
    );
    duration
}

pub fn run_benchmark(
    quack_ty: QuackType,
    num_trials: usize,
    num_packets: usize,
    params: QuackParams,
) {
    // Allocate buffer for benchmark durations.
    let mut durations: Vec<Duration> = vec![];

    for i in 0..(num_trials + 1) {
        let duration = match quack_ty {
            QuackType::Strawman1a => benchmark_construct_strawman1a(num_packets),
            QuackType::Strawman1b => benchmark_construct_strawman1b(params.threshold, num_packets),
            QuackType::Strawman2 => benchmark_construct_strawman2(num_packets),
            QuackType::PowerSum => {
                if params.precompute {
                    match params.num_bits_id {
                        16 => benchmark_construct_power_sum_precompute_u16(
                            params.threshold,
                            num_packets,
                        ),
                        32 => todo!(),
                        64 => todo!(),
                        _ => unimplemented!(),
                    }
                } else if params.montgomery {
                    match params.num_bits_id {
                        16 => unimplemented!(),
                        32 => unimplemented!(),
                        64 => benchmark_construct_power_sum_montgomery_u64(
                            params.threshold,
                            num_packets,
                        ),
                        _ => unimplemented!(),
                    }
                } else {
                    match params.num_bits_id {
                        16 => benchmark_construct_power_sum::<u16>(
                            params.threshold,
                            params.num_bits_id,
                            num_packets,
                        ),
                        32 => benchmark_construct_power_sum::<u32>(
                            params.threshold,
                            params.num_bits_id,
                            num_packets,
                        ),
                        64 => benchmark_construct_power_sum::<u64>(
                            params.threshold,
                            params.num_bits_id,
                            num_packets,
                        ),
                        _ => unimplemented!(),
                    }
                }
            }
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
    run_benchmark(args.quack_ty, args.num_trials, args.num_packets, args.quack);
}
