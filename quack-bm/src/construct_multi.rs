use crate::QuackParams;
use crate::common::*;

use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::ops::{Sub, SubAssign, AddAssign, MulAssign};
use std::time::{Instant, Duration};
use log::info;
use quack::{*, arithmetic::{ModularInteger, ModularArithmetic}};
use rand::distributions::{Standard, Distribution};

type AddrKey = [u8; 12];

fn benchmark_construct_power_sum<T>(
    size: usize,
    num_bits_id: usize,
    num_packets: usize,
    num_conns: usize,
) -> Duration
where Standard: Distribution<T>,
T: Debug + Display + Default + PartialOrd + Sub<Output = T> + Copy,
ModularInteger<T>: ModularArithmetic<T> + AddAssign + MulAssign + SubAssign {
    let numbers = gen_numbers::<T>(num_packets);
    let conns = gen_numbers::<AddrKey>(num_conns);
    let conn_numbers = gen_numbers::<usize>(num_packets)
        .into_iter()
        .enumerate()
        .map(|(i, index)| (conns[index % num_conns], numbers[i]))
        .collect::<Vec<(AddrKey, T)>>();

    // Construct an empty data structure for the quacks.
    let mut senders: HashMap<AddrKey, PowerSumQuack<T>> = HashMap::new();

    // Insert a bunch of random numbers into the accumulator.
    let t1 = Instant::now();
    for (conn, number) in conn_numbers.into_iter() {
        senders.entry(conn)
            .or_insert(PowerSumQuack::new(size))
            .insert(number);
    }
    let t2 = Instant::now();

    let duration = t2 - t1;
    info!("Insert {} numbers into {} Quacks (bits = {}, threshold = {}): {:?}",
        num_packets, num_conns, num_bits_id, size, duration);
    duration
}

pub fn run_benchmark(
    quack_ty: QuackType,
    num_trials: usize,
    num_packets: usize,
    num_conns: usize,
    params: QuackParams,
) {
    // Allocate buffer for benchmark durations.
    let mut durations: Vec<Duration> = vec![];

    for i in 0..(num_trials + 1) {
        assert_eq!(quack_ty, QuackType::PowerSum);
        assert!(!params.precompute);
        let duration = match params.num_bits_id {
            16 => benchmark_construct_power_sum::<u16>(
                params.threshold, params.num_bits_id, num_packets, num_conns),
            32 => benchmark_construct_power_sum::<u32>(
                params.threshold, params.num_bits_id, num_packets, num_conns),
            64 => benchmark_construct_power_sum::<u64>(
                params.threshold, params.num_bits_id, num_packets, num_conns),
            _ => unimplemented!(),
        };
        if i > 0 {
            durations.push(duration);
        }
    }
    print_summary(durations, num_packets);
}
