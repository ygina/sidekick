use crate::common::*;

use std::time::{Instant, Duration};
use log::info;
use rand::Rng;
use quack::*;

fn benchmark_decode_32(
    size: usize,
    num_packets: usize,
    num_drop: usize,
    num_trials: usize,
) {
    let mut rng = rand::thread_rng();

    // Allocate buffer for benchmark durations.
    let mut durations: Vec<Duration> = vec![];

    for i in 0..(num_trials + 1) {
        // Generate 1000 random numbers.
        let numbers: IdentifierLog =
            (0..num_packets).map(|_| rng.gen()).collect();

        // Construct two empty PowerSumQuacks.
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
        let dropped = DecodedQuack::decode(acc1, numbers);
        // do_not_discard(dropped);
        let t2 = Instant::now();

        if i > 0 {
            let duration = t2 - t1;
            info!("Decode time (u32, threshold = {}, num_packets={}, \
                false_positives = {}, dropped = {}): {:?}", size, num_packets,
                dropped.total_num_missing() - num_drop, num_drop, duration);
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
    benchmark_decode_32(threshold, num_packets, num_drop, num_trials);
}
