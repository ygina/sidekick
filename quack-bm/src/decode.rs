use crate::common::*;

use std::time::{Instant, Duration};
use log::info;
use rand::Rng;
use quack::*;

fn benchmark_decode_power_sum_32(
    numbers: Vec<u32>,
    factor: bool,
    size: usize,
    num_packets: usize,
    num_drop: usize,
) -> Duration {
    // Construct two empty Quacks.
    let mut acc1 = PowerSumQuack::new(size);
    let mut acc2 = PowerSumQuack::new(size);

    // Insert all random numbers into the first accumulator.
    for j in 0..num_packets {
        acc1.insert(numbers[j]);
    }

    // Insert all but num_drop random numbers into the second accumulator.
    for j in 0..(num_packets - num_drop) {
        acc2.insert(numbers[j]);
    }

    let t1 = Instant::now();
    acc1 -= acc2;
    let dropped = if factor {
        acc1.decode_by_factorization().unwrap()
    } else {
        acc1.decode_with_log(&numbers)
    };
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!("Decode time (u32, threshold = {}, num_packets={}, \
        false_positives = {}, dropped = {}): {:?}", size, num_packets,
        dropped.len() - num_drop, num_drop, duration);
    duration
}

pub fn run_benchmark(
    quack_ty: QuackType,
    use_tables: bool,
    factor: bool,
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
        // Generate 1000 random numbers.
        let numbers: IdentifierLog =
            (0..num_packets).map(|_| rng.gen()).collect();

        let duration = match quack_ty {
            QuackType::Strawman1 => unimplemented!(),
            QuackType::Strawman2 => unimplemented!(),
            QuackType::PowerSum => benchmark_decode_power_sum_32(
                numbers, factor, threshold, num_packets, num_drop),
            QuackType::Montgomery => unimplemented!(),
        };
        if i > 0 {
            durations.push(duration);
        }
    }
    print_summary(durations);
}
