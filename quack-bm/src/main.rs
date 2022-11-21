mod common;
mod construct;
mod decode;

use common::*;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    // Type of benchmark.
    #[arg(value_enum)]
    benchmark: BenchmarkType,
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
}


fn main() {
    let args = Cli::parse();
    match args.benchmark {
        BenchmarkType::Construct => {
            construct::run_benchmark(
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
                args.use_tables,
                args.threshold,
                args.num_packets,
                args.num_bits_id,
                args.num_drop,
                args.num_trials,
            )
        }
    }
}
