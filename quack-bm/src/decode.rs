use crate::common::*;

use std::time::{Instant, Duration};
use rand::Rng;
use quack::{Quack, arithmetic::{ModularInteger, MonicPolynomialEvaluator}};

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
        // Allocate variable for counting false positives.
        let mut fp = 0;

        // Generate 1000 random numbers.
        let numbers: Vec<u32> =
            (0..num_packets).map(|_| rng.gen()).collect();

        // Construct two empty Quacks.
        let mut acc1 = Quack::new(size);
        let mut acc2 = Quack::new(size);

        // Insert all random numbers into the first accumulator.
        for j in 0..num_packets {
            acc1.insert(numbers[j]);
        }

        // Insert all but num_drop random numbers into the second accumulator.
        for j in 0..(num_packets - num_drop) {
            acc2.insert(numbers[j]);
        }

        // Pre-allocate buffer for polynomial coefficients.
        let mut coeffs = (0..num_drop).map(|_| ModularInteger::zero()).collect();

        // Allocate buffer for missing packets.
        let mut dropped: Vec<u32> = vec![];

        let t1 = Instant::now();
        if num_drop > 0 {
            acc1 -= acc2;
            acc1.to_polynomial_coefficients(&mut coeffs);
            for j in 0..(num_packets - num_drop) {
                let value = MonicPolynomialEvaluator::eval(&coeffs, numbers[j]);
                if value.is_zero() {
                    fp += 1;
                }
            }
            for j in (num_packets - num_drop)..num_packets {
                let value = MonicPolynomialEvaluator::eval(&coeffs, numbers[j]);
                assert!(value.is_zero());
                dropped.push(numbers[j]);
            }
        }
        // do_not_discard(dropped);
        let t2 = Instant::now();

        if i > 0 {
            let duration = t2 - t1;
            println!("Decode time (u32, threshold = {}, num_packets={}, \
                false_positives = {}, dropped = {}): {:?}", size, num_packets,
                fp, num_drop, duration);
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
