use crate::QuackParams;
use crate::common::*;

use std::fmt::{Debug, Display};
use std::ops::{Sub, SubAssign, AddAssign, MulAssign};
use std::time::{Instant, Duration};
use log::info;
use quack::{*, arithmetic::{ModularInteger, ModularArithmetic}};
use rand::distributions::{Standard, Distribution};
use multiset::HashMultiSet;
use sha2::{Digest, Sha256};

fn benchmark_construct_strawman1(
    num_packets: usize,
) -> Duration {
    let numbers = gen_numbers::<u32>(num_packets);

    let mut acc = HashMultiSet::new();

    // Warm up the instruction cache by inserting a few numbers.
    for i in num_packets..(num_packets + 10) {
        acc.insert(numbers[i]);
    }

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for j in 0..num_packets {
        acc.insert(numbers[j]);
    }
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!("Insert {} numbers into 2 multisets: {:?}",
        num_packets, duration);
    duration
}

fn benchmark_construct_strawman2(
    num_packets: usize,
) -> Duration {
    let numbers = gen_numbers::<u32>(num_packets);
    let mut acc = Sha256::new();

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for i in 0..num_packets {
        acc.update(numbers[i].to_be_bytes());
    }
    acc.finalize();
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!("Insert {} numbers into a sha256 digest: {:?}",
        num_packets, duration);
    duration
}

fn benchmark_construct_power_sum_precompute_u16(
    size: usize,
    num_packets: usize,
) -> Duration {
    const WARMUP_PACKETS: usize = 10;
    let numbers = gen_numbers::<u16>(num_packets + WARMUP_PACKETS);

    // Construct two empty Quacks.
    let mut acc = PowerTableQuack::new(size);

    // Warm up the instruction cache by inserting a few numbers.
    for i in num_packets..(num_packets + WARMUP_PACKETS) {
        acc.insert(numbers[i]);
    }

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for j in 0..num_packets {
        acc.insert(numbers[j]);
    }
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!("Insert {} numbers into 2 Quacks (bits = 16, \
        threshold = {}): {:?}", num_packets, size, duration);
    duration
}

fn benchmark_construct_power_sum<T>(
    size: usize,
    num_bits_id: usize,
    num_packets: usize,
) -> Duration
where Standard: Distribution<T>,
T: Debug + Display + Default + PartialOrd + Sub<Output = T> + Copy,
ModularInteger<T>: ModularArithmetic<T> + AddAssign + MulAssign + SubAssign {
    const WARMUP_PACKETS: usize = 10;
    let numbers = gen_numbers::<T>(num_packets + WARMUP_PACKETS);

    // Construct two empty Quacks.
    let mut acc = PowerSumQuack::<T>::new(size);

    // Warm up the instruction cache by inserting a few numbers.
    for i in num_packets..(num_packets + WARMUP_PACKETS) {
        acc.insert(numbers[i]);
    }

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for j in 0..num_packets {
        acc.insert(numbers[j]);
    }
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!("Insert {} numbers into 2 Quacks (bits = {}, \
        threshold = {}): {:?}", num_packets, num_bits_id, size, duration);
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
            QuackType::Strawman1 => benchmark_construct_strawman1(num_packets),
            QuackType::Strawman2 => benchmark_construct_strawman2(num_packets),
            QuackType::PowerSum =>  if params.precompute {
                match params.num_bits_id {
                16 => benchmark_construct_power_sum_precompute_u16(params.threshold, num_packets),
                32 => todo!(),
                64 => todo!(),
                _ => unimplemented!(),
                }
            } else {
                match params.num_bits_id {
                16 => benchmark_construct_power_sum::<u16>(
                    params.threshold, params.num_bits_id, num_packets),
                32 => benchmark_construct_power_sum::<u32>(
                    params.threshold, params.num_bits_id, num_packets),
                64 => benchmark_construct_power_sum::<u64>(
                    params.threshold, params.num_bits_id, num_packets),
                _ => unimplemented!(),
                }
            },
        };
        if i > 0 {
            durations.push(duration);
        }
    }
    print_summary(durations, num_packets);
}
