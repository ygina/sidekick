# quack-bm

Figure 3 and Table 3.

```
$ ../target/release/quack-bm --help
Usage: quack-bm [OPTIONS] <BENCHMARK> <QUACK_TY>

Arguments:
  <BENCHMARK>  Type of benchmark [possible values: construct, decode]
  <QUACK_TY>   Quack type [possible values: strawman1, strawman2, power-sum]

Options:
      --trials <NUM_TRIALS>    Number of trials [default: 10]
  -n <NUM_PACKETS>             Number of sent packets [default: 1000]
  -d, --dropped <NUM_DROP>     Number of dropped packets [default: 20]
  -t, --threshold <THRESHOLD>  The threshold number of dropped packets [default: 20]
  -b, --bits <NUM_BITS_ID>     Number of identifier bits [default: 32]
      --precompute             Enable pre-computation optimization
      --factor                 Disable not-factoring optimization
      --montgomery             Enable Montgomery multiplication optimization
  -h, --help                   Print help information
```
