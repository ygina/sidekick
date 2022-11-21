use crate::common::*;

use std::time::{Instant, Duration};
use rand::Rng;
use quack::PowerSumAccumulator;

fn benchmark_construct_32(
    size: usize,
    num_packets: usize,
    num_drop: usize,
    num_trials: usize,
) {
    let mut rng = rand::thread_rng();

    // Allocate buffer for benchmark durations.
    let mut durations: Vec<Duration> = vec![];

    for i in 0..(num_trials + 1) {
        let numbers: Vec<u32> =
            (0..(num_packets + 10)).map(|_| rng.gen()).collect();

        // Construct two empty PowerSumAccumulators.
        let mut acc1 = PowerSumAccumulator::new(size);
        let mut acc2 = PowerSumAccumulator::new(size);

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

        if i > 0 {
            let duration = t2 - t1;
            println!("Insert {} numbers into 2 PowerSumAccumulators (u32, \
                threshold = {}): {:?}", num_packets, size, duration);
            durations.push(duration);
        }
    }
    print_summary(durations);
}

pub fn run_benchmark(
    use_tables: bool,
    threshold: usize,
    num_packets: usize,
    num_bits_id: usize,
    num_drop: usize,
    num_trials: usize,
) {
    assert!(!use_tables, "ERROR: power tables are not enabled");
    assert_eq!(num_bits_id, 32, "ERROR: <num_bits_id> must be 32");
    benchmark_construct_32(threshold, num_packets, num_drop, num_trials);
}
