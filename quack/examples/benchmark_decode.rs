use std::fmt::Debug;
use std::time::{Duration, Instant};

use clap::{Parser, ValueEnum};
use log::{info, warn};
use multiset::HashMultiSet;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use sha2::{Digest, Sha256};

use quack::*;

#[derive(Parser, Debug)]
pub struct QuackParams {
    /// The threshold number of dropped packets.
    #[arg(long, short = 't', default_value_t = 10)]
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
    #[arg(long = "trials", default_value_t = 1000)]
    num_trials: usize,
    /// Number of packets sent between quACKs.
    #[arg(short = 'n', default_value_t = 25)]
    num_packets: usize,
    /// Number of dropped packets.
    #[arg(short = 'd', long = "dropped", default_value_t = 10)]
    num_drop: usize,
    /// Quack parameters.
    #[command(flatten)]
    quack: QuackParams,
}

#[derive(Clone, ValueEnum, Debug, PartialEq, Eq)]
pub enum QuackType {
    Strawman1a,
    Strawman1b,
    Strawman2,
    PowerSum,
}

struct Timer {
    t: Instant,
    cycles: u64,
}

impl Timer {
    fn start() -> Timer {
        unsafe {
            core::arch::x86_64::_mm_lfence();
        }
        let t = Instant::now();
        let cycles = unsafe { core::arch::x86_64::_rdtsc() };
        unsafe {
            core::arch::x86_64::_mm_lfence();
        }
        Timer { t, cycles }
    }

    fn stop(&self) -> (Duration, u64) {
        unsafe {
            core::arch::x86_64::_mm_lfence();
        }
        let cycles = unsafe { core::arch::x86_64::_rdtsc() };
        let t = Instant::now();
        unsafe {
            core::arch::x86_64::_mm_lfence();
        }
        (t - self.t, cycles - self.cycles)
    }
}

pub fn print_summary(d: Vec<Duration>, cycles: Vec<u64>, num_packets: usize) {
    let size = d.len() as u32;
    let (avg_time, avg_cycles) = if d.is_empty() {
        (Duration::new(0, 0), 0)
    } else {
        let avg_time = d.into_iter().sum::<Duration>() / size;
        let avg_cycles = cycles.into_iter().sum::<u64>() / (size as u64);
        (avg_time, avg_cycles)
    };
    warn!(
        "SUMMARY: num_trials = {}, avg_cycles = {}, avg = {:?}",
        size, avg_cycles, avg_time,
    );
    let d_per_packet = avg_time / num_packets as u32;
    let cycles_per_packet = avg_cycles / num_packets as u64;
    let ns_per_packet = d_per_packet.as_secs() * 1000000000 + d_per_packet.subsec_nanos() as u64;
    let packets_per_s = 1000000000 / ns_per_packet;
    warn!(
        "SUMMARY (per-packet): {:?}/packet = {} packets/s = {} cycles/packet",
        d_per_packet, packets_per_s, cycles_per_packet,
    )
}

pub fn gen_numbers<T>(num_packets: usize) -> Vec<T>
where
    Standard: Distribution<T>,
{
    (0..num_packets).map(|_| rand::thread_rng().gen()).collect()
}

fn _benchmark_strawman1a(num_packets: usize, num_drop: usize) -> (Duration, u64) {
    let numbers = gen_numbers::<u32>(num_packets);

    // Construct two empty Quacks.
    let mut acc1 = HashMultiSet::new();
    let mut acc2 = HashMultiSet::new();

    // Insert all but num_drop random numbers into the second accumulator.
    for &number in numbers.iter().take(num_packets - num_drop) {
        acc2.insert(number);
    }

    let t = Timer::start();
    // Insert all random numbers into the first accumulator.
    // Then find the set difference.
    for &number in numbers.iter().take(num_packets) {
        acc1.insert(number);
    }
    let dropped = acc1 - acc2;
    let (duration, cycles) = t.stop();
    info!(
        "Decode time (num_packets={}, \
        false_positives = {}, dropped = {}): {:?}",
        num_packets,
        dropped.len() - num_drop,
        num_drop,
        duration
    );
    assert_eq!(dropped.len(), num_drop);
    (duration, cycles)
}

const NUM_SUBSETS_LIMIT: u32 = 1000000;

fn benchmark_strawman2(num_packets: usize, num_drop: usize) -> (Duration, u64) {
    let numbers = gen_numbers::<u32>(num_packets);
    let mut acc1 = Sha256::new();

    // Insert all but num_drop random numbers into the accumulator.
    for number in numbers.iter().take(num_packets - num_drop) {
        acc1.update(number.to_be_bytes());
    }
    acc1.finalize();

    // Calculate the number of subsets.
    let _n = num_packets as u32;
    let _r = num_drop as u32;
    // let num_subsets = (n-r+1..=n).product();

    let t = Timer::start();
    if num_drop > 0 {
        // For every subset of size "num_packets - num_drop"
        // Calculate the SHA256 hash
        // let num_hashes_to_calculate = std::cmp::min(
        //     NUM_SUBSETS_LIMIT, num_subsets / 2);
        let num_hashes_to_calculate = NUM_SUBSETS_LIMIT;

        // We're really just measuring a lower bound of the time to compute
        // any SHA256 hash with this number of elements
        for _ in 0..num_hashes_to_calculate {
            let mut acc2 = Sha256::new();
            for number in numbers.iter().take(num_packets - num_drop) {
                acc2.update(number.to_be_bytes());
            }
            acc2.finalize();
        }
    }
    let (duration, cycles) = t.stop();
    info!("Decode time (num_packets={}): {:?}", num_packets, duration);
    info!(
        "Calculated {} hashes, expected {}C{}",
        NUM_SUBSETS_LIMIT, num_packets, num_drop
    );
    (duration, cycles)
}

fn benchmark_factor_u32(threshold: usize, num_packets: usize, num_drop: usize) -> (Duration, u64) {
    let numbers = gen_numbers::<u32>(num_packets);

    // Construct two empty Quacks.
    let mut acc1 = PowerSumQuackU32::new(threshold);
    let mut acc2 = PowerSumQuackU32::new(threshold);

    // Insert all but num_drop random numbers into the second accumulator.
    for &number in numbers.iter().take(num_packets - num_drop) {
        acc2.insert(number);
    }

    let t = Timer::start();
    for &number in numbers.iter().take(num_packets) {
        acc1.insert(number);
    }
    acc1.sub_assign(acc2);
    let dropped = acc1.decode_by_factorization().unwrap();
    let (duration, cycles) = t.stop();
    info!(
        "Decode time PowerSumQuackU32 + factor (threshold = {}, num_packets={}, \
        false_positives = {}, dropped = {}): {:?}",
        threshold,
        num_packets,
        dropped.len() - num_drop,
        num_drop,
        duration
    );
    assert_eq!(dropped.len(), num_drop);
    (duration, cycles)
}

fn benchmark<T: PowerSumQuack>(
    mut acc1: T,
    mut acc2: T,
    name: &str,
    num_packets: usize,
    num_drop: usize,
) -> (Duration, u64)
where
    Standard: Distribution<<T as PowerSumQuack>::Element>,
    <T as PowerSumQuack>::Element: Copy,
{
    let numbers = gen_numbers(num_packets);

    // Insert all but num_drop random numbers into the second accumulator.
    for &number in numbers.iter().take(num_packets - num_drop) {
        acc2.insert(number);
    }

    let t = Timer::start();
    for &number in numbers.iter().take(num_packets) {
        acc1.insert(number);
    }
    acc1.sub_assign(acc2);
    let dropped = acc1.decode_with_log(&numbers);
    let (duration, cycles) = t.stop();
    info!(
        "Decode time {} (threshold = {}, num_packets={}, \
        false_positives = {}, dropped = {}): {:?}",
        name,
        acc1.threshold(),
        num_packets,
        dropped.len() - num_drop,
        num_drop,
        duration
    );
    assert!(dropped.len() >= num_drop);
    (duration, cycles)
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();
    let n = args.num_packets;
    let t = args.quack.threshold;
    let b = args.quack.num_bits_id;
    let m = args.num_drop;

    quack::global_config_set_max_power_sum_threshold(args.quack.threshold);

    let mut durations: Vec<Duration> = vec![];
    let mut cycles_vec: Vec<u64> = vec![];
    for i in 0..(args.num_trials + 1) {
        let (duration, cycles) = match args.quack_ty {
            QuackType::Strawman1a => {
                unimplemented!("decoding R is trivial because the identifiers are unmodified")
            }
            QuackType::Strawman1b => {
                unimplemented!("decoding R is trivial because the identifiers are unmodified")
            }
            QuackType::Strawman2 => benchmark_strawman2(n, m),
            QuackType::PowerSum => {
                if b == 16 {
                    assert!(!args.quack.montgomery);
                    assert!(!args.quack.factor);
                    if args.quack.precompute {
                        benchmark(
                            PowerTableQuack::new(t),
                            PowerTableQuack::new(t),
                            "PowerTableQuack",
                            n,
                            m,
                        )
                    } else {
                        benchmark(
                            PowerSumQuackU16::new(t),
                            PowerSumQuackU16::new(t),
                            "PowerSumQuackU16",
                            n,
                            m,
                        )
                    }
                } else if b == 32 {
                    assert!(!args.quack.montgomery);
                    assert!(!args.quack.precompute);
                    if args.quack.factor {
                        benchmark_factor_u32(t, n, m)
                    } else {
                        benchmark(
                            PowerSumQuackU32::new(t),
                            PowerSumQuackU32::new(t),
                            "PowerSumQuackU32",
                            n,
                            m,
                        )
                    }
                } else if b == 64 {
                    assert!(!args.quack.precompute);
                    assert!(!args.quack.factor);
                    if args.quack.montgomery {
                        benchmark(
                            MontgomeryQuack::new(t),
                            MontgomeryQuack::new(t),
                            "MontgomeryQuack",
                            n,
                            m,
                        )
                    } else {
                        benchmark(
                            PowerSumQuackU64::new(t),
                            PowerSumQuackU64::new(t),
                            "PowerSumQuackU64",
                            n,
                            m,
                        )
                    }
                } else {
                    unimplemented!("no other bit widths supported");
                }
            }
        };
        if i != 0 {
            durations.push(duration);
            cycles_vec.push(cycles);
        }
    }
    print_summary(durations, cycles_vec, n)
}
