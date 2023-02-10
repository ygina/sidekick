use crate::common::*;

use std::time::{Instant, Duration};
use log::info;
use rand::Rng;
use quack::*;
use multiset::HashMultiSet;
use sha2::{Digest, Sha256};

fn benchmark_decode_strawman1(
    numbers: Vec<u32>,
    num_packets: usize,
    num_drop: usize,
) -> Duration {
    // Construct two empty Quacks.
    let mut acc1 = HashMultiSet::new();
    let mut acc2 = HashMultiSet::new();

    // Insert all random numbers into the first accumulator.
    for j in 0..num_packets {
        acc1.insert(numbers[j]);
    }

    // Insert all but num_drop random numbers into the second accumulator.
    for j in 0..(num_packets - num_drop) {
        acc2.insert(numbers[j]);
    }

    let t1 = Instant::now();
    let dropped = acc1 - acc2;
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!("Decode time (num_packets={}, \
        false_positives = {}, dropped = {}): {:?}", num_packets,
        dropped.len() - num_drop, num_drop, duration);
    assert_eq!(dropped.len(), num_drop);
    duration
}

const NUM_SUBSETS_LIMIT: u32 = 1000000;

fn benchmark_decode_strawman2(
    numbers: Vec<u32>,
    num_packets: usize,
    num_drop: usize,
) -> Duration {
    let mut acc1 = Sha256::new();

    // Insert all but num_drop random numbers into the accumulator.
    for i in 0..(num_packets - num_drop) {
        acc1.update(numbers[i].to_be_bytes());
    }
    acc1.finalize();

    // Calculate the number of subsets.
    let n = num_packets as u32;
    let r = num_drop as u32;
    // let num_subsets = (n-r+1..=n).product();

    let t1 = Instant::now();
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
            for j in 0..(num_packets - num_drop) {
                acc2.update(numbers[j].to_be_bytes());
            }
            acc2.finalize();
        }
    }
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!("Decode time (num_packets={}): {:?}", num_packets, duration);
    info!("Calculated {} hashes, expected {}C{}",
        NUM_SUBSETS_LIMIT, num_packets, num_drop);

    duration
}

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
    assert_eq!(dropped.len(), num_drop);
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
            QuackType::Strawman1 => benchmark_decode_strawman1(
                numbers, num_packets, num_drop),
            QuackType::Strawman2 => benchmark_decode_strawman2(
                numbers, num_packets, num_drop),
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
