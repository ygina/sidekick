use crate::QuackParams;
use crate::common::*;

use std::fmt::{Debug, Display};
use std::ops::{Sub, SubAssign, AddAssign, MulAssign};
use std::time::{Instant, Duration};
use log::info;
use rand::distributions::{Standard, Distribution};
use quack::{*, arithmetic::{ModularInteger, ModularArithmetic}};
use multiset::HashMultiSet;
use sha2::{Digest, Sha256};

fn benchmark_decode_strawman1(
    num_packets: usize,
    num_drop: usize,
) -> Duration {
    let numbers = gen_numbers::<u32>(num_packets);

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
    num_packets: usize,
    num_drop: usize,
) -> Duration {
    let numbers = gen_numbers::<u32>(num_packets);
    let mut acc1 = Sha256::new();

    // Insert all but num_drop random numbers into the accumulator.
    for i in 0..(num_packets - num_drop) {
        acc1.update(numbers[i].to_be_bytes());
    }
    acc1.finalize();

    // Calculate the number of subsets.
    let _n = num_packets as u32;
    let _r = num_drop as u32;
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

fn benchmark_decode_power_sum_factor_u32(
    size: usize,
    num_packets: usize,
    num_drop: usize,
) -> Duration {
    let numbers = gen_numbers::<u32>(num_packets);

    // Construct two empty Quacks.
    let mut acc1 = PowerSumQuack::<u32>::new(size);
    let mut acc2 = PowerSumQuack::<u32>::new(size);

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
    let dropped = acc1.decode_by_factorization().unwrap();
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!("Decode time (bits = 32, threshold = {}, num_packets={}, \
        false_positives = {}, dropped = {}): {:?}", size,
        num_packets, dropped.len() - num_drop, num_drop, duration);
    assert_eq!(dropped.len(), num_drop);
    duration
}

fn benchmark_decode_power_sum_precompute_u16(
    size: usize,
    num_packets: usize,
    num_drop: usize,
) -> Duration {
    let numbers = gen_numbers::<u16>(num_packets);

    // Construct two empty Quacks.
    let mut acc1 = PowerTableQuack::new(size);
    let mut acc2 = PowerTableQuack::new(size);

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
    let dropped = acc1.decode_with_log(&numbers);
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!("Decode time (bits = 32, threshold = {}, num_packets={}, \
        false_positives = {}, dropped = {}): {:?}", size,
        num_packets, dropped.len() - num_drop, num_drop, duration);
    assert!(dropped.len() >= num_drop);
    duration
}

fn benchmark_decode_power_sum<T>(
    size: usize,
    num_bits_id: usize,
    num_packets: usize,
    num_drop: usize,
) -> Duration
where Standard: Distribution<T>,
T: Debug + Display + Default + PartialOrd + Sub<Output = T> + Copy,
ModularInteger<T>: ModularArithmetic<T> + AddAssign + MulAssign + SubAssign {
    let numbers = gen_numbers::<T>(num_packets);

    // Construct two empty Quacks.
    let mut acc1 = PowerSumQuack::<T>::new(size);
    let mut acc2 = PowerSumQuack::<T>::new(size);

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
    let dropped = acc1.decode_with_log(&numbers);
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!("Decode time (bits = {}, threshold = {}, num_packets={}, \
        false_positives = {}, dropped = {}): {:?}", num_bits_id, size,
        num_packets, dropped.len() - num_drop, num_drop, duration);
    assert!(dropped.len() >= num_drop);
    duration
}

pub fn run_benchmark(
    quack_ty: QuackType,
    num_trials: usize,
    num_packets: usize,
    num_drop: usize,
    params: QuackParams,
) {
    // Allocate buffer for benchmark durations.
    let mut durations: Vec<Duration> = vec![];

    for i in 0..(num_trials + 1) {
        let duration = match quack_ty {
            QuackType::Strawman1 => benchmark_decode_strawman1(num_packets, num_drop),
            QuackType::Strawman2 => benchmark_decode_strawman2(num_packets, num_drop),
            QuackType::PowerSum => if params.factor {
                match params.num_bits_id {
                16 => todo!(),
                32 => benchmark_decode_power_sum_factor_u32(params.threshold, num_packets, num_drop),
                64 => todo!(),
                _ => unimplemented!(),
                }
            } else if params.precompute {
                match params.num_bits_id {
                16 => benchmark_decode_power_sum_precompute_u16(params.threshold, num_packets, num_drop),
                32 => todo!(),
                64 => todo!(),
                _ => unimplemented!(),
                }
            } else {
                match params.num_bits_id {
                16 => benchmark_decode_power_sum::<u16>(params.threshold, params.num_bits_id, num_packets, num_drop),
                32 => benchmark_decode_power_sum::<u32>(params.threshold, params.num_bits_id, num_packets, num_drop),
                64 => benchmark_decode_power_sum::<u64>(params.threshold, params.num_bits_id, num_packets, num_drop),
                _ => unimplemented!(),
                }
            }
        };
        if i > 0 {
            durations.push(duration);
        }
    }
    print_summary(durations, num_packets);
}
