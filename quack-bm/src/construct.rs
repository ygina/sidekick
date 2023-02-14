use crate::common::*;

use std::time::{Instant, Duration};
use log::info;
use rand::Rng;
use quack::*;
use multiset::HashMultiSet;
use sha2::{Digest, Sha256};

fn benchmark_construct_strawman1(
    numbers: Vec<u32>,
    num_packets: usize,
    num_drop: usize,
) -> Duration {
    let mut acc1 = HashMultiSet::new();
    let mut acc2 = HashMultiSet::new();

    // Warm up the instruction cache by inserting a few numbers.
    for i in num_packets..(num_packets + 10) {
        acc1.insert(numbers[i]);
    }
    for i in num_packets..(num_packets + 10) {
        acc2.insert(numbers[i]);
    }

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for j in 0..num_packets {
        acc1.insert(numbers[j]);
    }
    for j in 0..(num_packets - num_drop) {
        acc2.insert(numbers[j]);
    }
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!("Insert {} numbers into 2 multisets: {:?}",
        num_packets, duration);
    duration
}

fn benchmark_construct_strawman2(
    numbers: Vec<u32>,
    num_packets: usize,
    num_drop: usize,
) -> Duration {
    let mut acc = Sha256::new();

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for i in 0..(num_packets - num_drop) {
        acc.update(numbers[i].to_be_bytes());
    }
    acc.finalize();
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!("Insert {} numbers into a sha256 digest: {:?}",
        num_packets, duration);
    duration
}

fn benchmark_construct_power_sum_32(
    numbers: Vec<u32>,
    size: usize,
    num_packets: usize,
    num_drop: usize,
) -> Duration {
    // Construct two empty Quacks.
    let mut acc1 = PowerSumQuack::new(size);
    let mut acc2 = PowerSumQuack::new(size);

    // Warm up the instruction cache by inserting a few numbers.
    for i in num_packets..(num_packets + 10) {
        acc1.insert(numbers[i]);
    }
    for i in num_packets..(num_packets + 10) {
        acc2.insert(numbers[i]);
    }

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for j in 0..num_packets {
        acc1.insert(numbers[j]);
    }
    for j in 0..(num_packets - num_drop) {
        acc2.insert(numbers[j]);
    }
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!("Insert {} numbers into 2 Quacks (u32, \
        threshold = {}): {:?}", num_packets, size, duration);
    duration
}

pub fn run_benchmark(
    quack_ty: QuackType,
    use_tables: bool,
    threshold: usize,
    num_packets: usize,
    num_bits_id: usize,
    num_drop: usize,
    num_trials: usize,
) {
    assert!(!use_tables, "ERROR: power tables are not enabled");
    assert_eq!(num_bits_id, 32, "ERROR: <num_bits_id> must be 32");

    let mut rng = rand::thread_rng();

    // Allocate buffer for benchmark durations.
    let mut durations: Vec<Duration> = vec![];

    for i in 0..(num_trials + 1) {
        let numbers: Vec<u32> =
            (0..(num_packets + 10)).map(|_| rng.gen()).collect();

        let duration = match quack_ty {
            QuackType::Strawman1 => benchmark_construct_strawman1(
                numbers, num_packets, num_drop),
            QuackType::Strawman2 => benchmark_construct_strawman2(
                numbers, num_packets, num_drop),
            QuackType::PowerSum => benchmark_construct_power_sum_32(
                numbers, threshold, num_packets, num_drop),
        };
        if i > 0 {
            durations.push(duration);
        }
    }
    print_summary(durations);
}
