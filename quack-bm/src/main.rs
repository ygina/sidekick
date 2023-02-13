mod common;
mod construct;
mod decode;

use quack::arithmetic::init_pow_table;
use common::*;
use clap::Parser;
use log::debug;

#[derive(Parser, Debug)]
struct Cli {
    // Type of benchmark.
    #[arg(value_enum)]
    benchmark: BenchmarkType,
    // Quack type.
    #[arg(value_enum)]
    quack_ty: QuackType,
    // The threshold number of dropped packets.
    #[arg(short = 't', default_value_t = 20)]
    threshold: usize,
    // Number of sent packets.
    #[arg(short = 'n', default_value_t = 1000)]
    num_packets: usize,
    // Number of identifier bits.
    #[arg(short = 'b', default_value_t = 32)]
    num_bits_id: usize,
    // Number of dropped packets.
    #[arg(long = "dropped", default_value_t = 20)]
    num_drop: usize,
    // Number of trials.
    #[arg(long = "trials", default_value_t = 10)]
    num_trials: usize,
    // Whether to use power tables.
    #[arg(long = "use-tables")]
    use_tables: bool,
    // Whether to factor if using power sum quacks.
    #[arg(long = "factor")]
    factor: bool,
}


fn main() {
    env_logger::init();
    init_pow_table();

    let args = Cli::parse();
    debug!("args = {:?}", args);
    match args.benchmark {
        BenchmarkType::Construct => {
            construct::run_benchmark(
                args.quack_ty,
                args.use_tables,
                args.threshold,
                args.num_packets,
                args.num_bits_id,
                args.num_drop,
                args.num_trials,
            )
        }
        BenchmarkType::Decode => {
            decode::run_benchmark(
                args.quack_ty,
                args.use_tables,
                args.factor,
                args.threshold,
                args.num_packets,
                args.num_bits_id,
                args.num_drop,
                args.num_trials,
            )
        }
    }
}
