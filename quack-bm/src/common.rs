use clap::ValueEnum;
use std::time::Duration;
use log::info;

#[derive(Clone, ValueEnum, Debug)]
pub enum BenchmarkType {
    Construct,
    Decode,
}

#[derive(Clone, ValueEnum, Debug)]
pub enum QuackType {
    Strawman1,
    Strawman2,
    PowerSum,
}

pub fn print_summary(d: Vec<Duration>) {
    let size = d.len() as u32;
    let avg = if d.is_empty() {
        Duration::new(0, 0)
    } else {
        d.into_iter().sum::<Duration>() / size
    };
    info!("SUMMARY: num_trials = {}, avg = {:?}", size, avg);
}
