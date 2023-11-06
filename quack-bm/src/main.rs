mod common;
mod construct;
mod construct_multi;
mod decode;

use clap::Parser;
use common::*;
use log::debug;

#[derive(Parser, Debug)]
pub struct QuackParams {
    /// The threshold number of dropped packets.
    #[arg(long, short = 't', default_value_t = 20)]
    threshold: usize,
    /// Number of identifier bits.
    #[arg(long = "bits", short = 'b', default_value_t = 32)]
    num_bits_id: usize,
    /// Enable pre-computation optimization
    #[arg(long)]
    precompute: bool,
    /// Disable not-factoring optimization
    #[arg(long)]
    factor: bool,
    /// Enable Montgomery multiplication optimization
    #[arg(long)]
    montgomery: bool,
}

#[derive(Parser, Debug)]
struct Cli {
    /// Type of benchmark.
    #[arg(value_enum)]
    benchmark: BenchmarkType,
    /// Quack type.
    #[arg(value_enum)]
    quack_ty: QuackType,
    /// Number of trials.
    #[arg(long = "trials", default_value_t = 10)]
    num_trials: usize,
    /// Number of sent packets.
    #[arg(short = 'n', default_value_t = 1000)]
    num_packets: usize,
    /// Number of dropped packets.
    #[arg(short = 'd', long = "dropped", default_value_t = 20)]
    num_drop: usize,
    /// Number of connections.
    #[arg(short = 'c', long = "connections", default_value_t = 1)]
    num_conns: usize,
    /// Quack parameters.
    #[command(flatten)]
    quack: QuackParams,
}

fn main() {
    env_logger::init();

    let args = Cli::parse();
    debug!("args = {:?}", args);
    match args.benchmark {
        BenchmarkType::Construct => {
            construct::run_benchmark(args.quack_ty, args.num_trials, args.num_packets, args.quack)
        }
        BenchmarkType::ConstructMulti => construct_multi::run_benchmark(
            args.quack_ty,
            args.num_trials,
            args.num_packets,
            args.num_conns,
            args.quack,
        ),
        BenchmarkType::Decode => decode::run_benchmark(
            args.quack_ty,
            args.num_trials,
            args.num_packets,
            args.num_drop,
            args.quack,
        ),
    }
}
