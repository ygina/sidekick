use clap::ValueEnum;
use std::time::Duration;

#[derive(Clone, ValueEnum)]
pub enum BenchmarkType {
    Construct,
    Decode,
}

pub fn print_summary(d: Vec<Duration>) {
    let size = d.len() as u32;
    let avg = if d.is_empty() {
        Duration::new(0, 0)
    } else {
        d.into_iter().sum::<Duration>() / size
    };
    println!("SUMMARY: num_trials = {}, avg = {:?}", size, avg);
}
