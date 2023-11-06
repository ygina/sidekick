use clap::ValueEnum;
use log::warn;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::time::Duration;

#[derive(Clone, ValueEnum, Debug)]
pub enum BenchmarkType {
    Construct,
    ConstructMulti,
    Decode,
}

#[derive(Clone, ValueEnum, Debug, PartialEq, Eq)]
pub enum QuackType {
    Strawman1a,
    Strawman1b,
    Strawman2,
    PowerSum,
}

pub fn print_summary(d: Vec<Duration>, num_packets: usize) {
    let size = d.len() as u32;
    let avg = if d.is_empty() {
        Duration::new(0, 0)
    } else {
        d.into_iter().sum::<Duration>() / size
    };
    warn!("SUMMARY: num_trials = {}, avg = {:?}", size, avg);
    let d_per_packet = avg / num_packets as u32;
    let ns_per_packet = d_per_packet.as_secs() * 1000000000 + d_per_packet.subsec_nanos() as u64;
    let packets_per_s = 1000000000 / ns_per_packet;
    warn!(
        "SUMMARY (per-packet): {:?}/packet = {} packets/s",
        d_per_packet, packets_per_s
    )
}

pub fn gen_numbers<T>(num_packets: usize) -> Vec<T>
where
    Standard: Distribution<T>,
{
    (0..num_packets).map(|_| rand::thread_rng().gen()).collect()
}
