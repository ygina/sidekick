use crate::common::*;
use crate::QuackParams;

use bincode;
use log::info;
use quack::{
    arithmetic::{ModularArithmetic, ModularInteger},
    *,
};
use rand::distributions::{Distribution, Standard};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt::{Debug, Display};
use std::ops::{AddAssign, MulAssign, Sub, SubAssign};
use std::time::{Duration, Instant};

fn benchmark_construct_strawman1a(num_packets: usize) -> Duration {
    let numbers = gen_numbers::<u32>(num_packets);

    let mut quack = StrawmanAQuack { sidecar_id: 0 };

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for number in numbers {
        quack.sidecar_id = number;
        let _bytes = bincode::serialize(&quack).unwrap();
    }
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!(
        "Serialize {} numbers into StrawmanAQuack: {:?}",
        num_packets, duration
    );
    duration
}

fn benchmark_construct_strawman1b(threshold: usize, num_packets: usize) -> Duration {
    let numbers = gen_numbers::<u32>(num_packets);

    let mut quack = StrawmanBQuack::new(threshold);

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for number in numbers {
        quack.insert(number);
        let _bytes = bincode::serialize(&quack).unwrap();
    }
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!(
        "Serialize {} numbers into StrawmanBQuack with threshold {}: {:?}",
        num_packets, threshold, duration
    );
    duration
}

fn benchmark_construct_strawman2(num_packets: usize) -> Duration {
    let numbers = gen_numbers::<u32>(num_packets);
    let mut acc = Sha256::new();

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for i in 0..num_packets {
        acc.update(numbers[i].to_be_bytes());
    }
    let _array = acc.finalize();
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!(
        "Insert {} numbers into a sha256 digest: {:?}",
        num_packets, duration
    );
    duration
}

fn benchmark_construct_power_sum_precompute_u16(threshold: usize, num_packets: usize) -> Duration {
    let numbers = gen_numbers::<u16>(num_packets);

    // Construct two empty Quacks.
    let mut quack = PowerTableQuack::new(threshold);

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for number in numbers {
        quack.insert(number);
    }
    let _bytes = bincode::serialize(&quack);
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!(
        "Insert {} numbers into 2 Quacks (bits = 16, \
        threshold = {}): {:?}",
        num_packets, threshold, duration
    );
    duration
}

fn benchmark_construct_power_sum_montgomery_u64(threshold: usize, num_packets: usize) -> Duration {
    let numbers = gen_numbers::<u64>(num_packets);

    // Construct an empty Quack.
    let mut quack = MontgomeryQuack::new(threshold);

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for number in numbers {
        quack.insert(number);
    }
    let _bytes = bincode::serialize(&quack);
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!(
        "Insert {} numbers into a montgomery quACK (bits = 64, \
        threshold = {}): {:?}",
        num_packets, threshold, duration
    );
    duration
}

fn benchmark_construct_power_sum<T>(
    threshold: usize,
    num_bits_id: usize,
    num_packets: usize,
) -> Duration
where
    Standard: Distribution<T>,
    T: Debug + Display + Default + PartialOrd + Sub<Output = T> + Copy + Serialize,
    ModularInteger<T>: ModularArithmetic<T> + AddAssign + MulAssign + SubAssign,
{
    let numbers = gen_numbers::<T>(num_packets);

    // Construct two empty Quacks.
    let mut quack = PowerSumQuack::<T>::new(threshold);

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    #[cfg(feature = "cycles")]
    let start = unsafe { core::arch::x86_64::_rdtsc() };
    for number in numbers {
        quack.insert(number);
    }
    #[cfg(feature = "cycles")]
    let end = unsafe { core::arch::x86_64::_rdtsc() };
    let _bytes = bincode::serialize(&quack);
    let t2 = Instant::now();

    let duration = t2 - t1;
    #[cfg(not(feature = "cycles"))]
    info!(
        "Insert {} numbers into a power sum quACK (bits = {}, \
        threshold = {}): {:?}",
        num_packets, num_bits_id, threshold, duration
    );
    #[cfg(feature = "cycles")]
    info!(
        "Insert {} numbers into a power sum quACK (bits = {}, \
        threshold = {}): {:?} ({} cycles/pkt)",
        num_packets,
        num_bits_id,
        threshold,
        duration,
        (end - start) / (num_packets as u64)
    );
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
            QuackType::Strawman1a => benchmark_construct_strawman1a(num_packets),
            QuackType::Strawman1b => benchmark_construct_strawman1b(params.threshold, num_packets),
            QuackType::Strawman2 => benchmark_construct_strawman2(num_packets),
            QuackType::PowerSum => {
                if params.precompute {
                    match params.num_bits_id {
                        16 => benchmark_construct_power_sum_precompute_u16(
                            params.threshold,
                            num_packets,
                        ),
                        32 => todo!(),
                        64 => todo!(),
                        _ => unimplemented!(),
                    }
                } else if params.montgomery {
                    match params.num_bits_id {
                        16 => unimplemented!(),
                        32 => unimplemented!(),
                        64 => benchmark_construct_power_sum_montgomery_u64(
                            params.threshold,
                            num_packets,
                        ),
                        _ => unimplemented!(),
                    }
                } else {
                    match params.num_bits_id {
                        16 => benchmark_construct_power_sum::<u16>(
                            params.threshold,
                            params.num_bits_id,
                            num_packets,
                        ),
                        32 => benchmark_construct_power_sum::<u32>(
                            params.threshold,
                            params.num_bits_id,
                            num_packets,
                        ),
                        64 => benchmark_construct_power_sum::<u64>(
                            params.threshold,
                            params.num_bits_id,
                            num_packets,
                        ),
                        _ => unimplemented!(),
                    }
                }
            }
        };
        if i > 0 {
            durations.push(duration);
        }
    }
    print_summary(durations, num_packets);
}
