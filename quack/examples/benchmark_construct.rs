use std::fmt::Debug;
use std::time::{Duration, Instant};

use clap::{Parser, ValueEnum};
use log::{info, warn};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use quack::*;

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
    #[arg(long, short = 't', default_value_t = 10)]
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
    #[arg(long = "trials", default_value_t = 1000)]
    num_trials: usize,
    /// Number of encoded packets.
    #[arg(short = 'e', default_value_t = 1000)]
    num_packets: usize,
    /// Quack parameters.
    #[command(flatten)]
    quack: QuackParams,
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
        size, avg_cycles, avg_time
    );
    let d_per_packet = avg_time / num_packets as u32;
    let cycles_per_packet = avg_cycles / num_packets as u64;
    let ns_per_packet = d_per_packet.as_secs() * 1000000000 + d_per_packet.subsec_nanos() as u64;
    let packets_per_s = if ns_per_packet == 0 {
        "NaN".to_string()
    } else {
        (1000000000 / ns_per_packet).to_string()
    };
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

fn benchmark_strawman1a(num_packets: usize) -> (Duration, u64) {
    let numbers = gen_numbers::<u32>(num_packets);

    let mut quack = StrawmanAQuack { sidecar_id: 0 };

    // Insert a bunch of random numbers into the accumulator.
    let t = Timer::start();
    for number in numbers {
        quack.sidecar_id = number;
        let _bytes = bincode::serialize(&quack).unwrap();
    }

    let (duration, cycles) = t.stop();
    info!(
        "Serialize {} numbers into StrawmanAQuack: {:?} ({} cycles/pkt)",
        num_packets,
        duration,
        cycles / (num_packets as u64),
    );
    (duration, cycles)
}

fn benchmark_strawman1b(threshold: usize, num_packets: usize) -> (Duration, u64) {
    let numbers = gen_numbers::<u32>(num_packets);

    let mut quack = StrawmanBQuack::new(threshold);

    // Insert a bunch of random numbers into the accumulator.
    let t = Timer::start();
    for number in numbers {
        quack.insert(number);
        let _bytes = bincode::serialize(&quack).unwrap();
    }

    let (duration, cycles) = t.stop();
    info!(
        "Serialize {} numbers into StrawmanBQuack with threshold {}: {:?} ({} cycles/pkt)",
        num_packets,
        threshold,
        duration,
        cycles / (num_packets as u64),
    );
    (duration, cycles)
}

fn benchmark_strawman2(num_packets: usize) -> (Duration, u64) {
    let numbers = gen_numbers::<u32>(num_packets);
    let mut acc = Sha256::new();

    // Insert a bunch of random numbers into the accumulator.
    let t = Timer::start();
    for number in numbers.iter().take(num_packets) {
        acc.update(number.to_be_bytes());
    }
    let _array = acc.finalize();

    let (duration, cycles) = t.stop();
    info!(
        "Insert {} numbers into a sha256 digest: {:?} ({} cycles/pkt)",
        num_packets,
        duration,
        cycles / (num_packets as u64),
    );
    (duration, cycles)
}

fn benchmark<T: PowerSumQuack + Serialize>(
    mut quack: T,
    name: &str,
    num_packets: usize,
) -> (Duration, u64)
where
    Standard: Distribution<<T as PowerSumQuack>::Element>,
{
    let numbers = gen_numbers(num_packets);

    // Insert a bunch of random numbers into the accumulator.
    let t = Timer::start();
    for number in numbers {
        quack.insert(number);
    }
    let _bytes = bincode::serialize(&quack);

    let (duration, cycles) = t.stop();
    info!(
        "Insert {} numbers into a {} (threshold = {}): {:?} ({} cycles/pkt)",
        num_packets,
        name,
        quack.threshold(),
        duration,
        cycles / (num_packets as u64)
    );
    (duration, cycles)
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();
    let n = args.num_packets;
    let t = args.quack.threshold;
    let b = args.quack.num_bits_id;

    quack::global_config_set_max_power_sum_threshold(args.quack.threshold);

    let mut durations: Vec<Duration> = vec![];
    let mut cycles_vec: Vec<u64> = vec![];
    for i in 0..(args.num_trials + 1) {
        let (duration, cycles) = match args.quack_ty {
            QuackType::Strawman1a => benchmark_strawman1a(n),
            QuackType::Strawman1b => benchmark_strawman1b(t, n),
            QuackType::Strawman2 => benchmark_strawman2(n),
            QuackType::PowerSum => {
                if b == 16 {
                    assert!(!args.quack.montgomery);
                    if args.quack.precompute {
                        benchmark(PowerTableQuack::new(t), "PowerTableQuack", n)
                    } else {
                        benchmark(PowerSumQuackU16::new(t), "PowerSumQuackU16", n)
                    }
                } else if b == 32 {
                    assert!(!args.quack.montgomery);
                    assert!(!args.quack.precompute);
                    benchmark(PowerSumQuackU32::new(t), "PowerSumQuackU32", n)
                } else if b == 64 {
                    assert!(!args.quack.precompute);
                    if args.quack.montgomery {
                        benchmark(MontgomeryQuack::new(t), "MontgomeryQuack", n)
                    } else {
                        benchmark(PowerSumQuackU64::new(t), "PowerSumQuackU64", n)
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
