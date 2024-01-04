# NSDI '24 Experiments

Each figure in the paper has a corresponding script. The script parses log files
in an output directory and plots the data. If the log files are missing data,
the script executes the experiments and adds the data to the log files.

## Dependencies

Setup a Python virtual environment and install plotting dependencies.

```
python3 -m venv env
source env/bin/activate
pip install -r requirements.txt
```

## Table 3

```
./target/release/benchmark_construct [strawman1a|strawman1b|strawman2|power-sum] -n 1000 -t 20 -b 32
./target/release/benchmark_decode [strawman1a|strawman1b|strawman2|power-sum] -n 1000 -t 20 -b 32
```

## Figure 2

```
python figures/fig2_microbenchmarks.py --logdir $HOME/sidecar/nsdi --execute
```

## Figure 4

```
python figures/fig4a_pep_emulation.py --marquee --logdir $HOME/sidecar/nsdi --execute [-t 1]
python figures/fig4b_low_latency_media.py --logdir $HOME/sidecar/nsdi --execute [-t 1]
python figures/fig4c_ack_reduction.py --marquee --logdir $HOME/sidecar/nsdi --execute
```

## Figure 5

```
python figures/fig5_baseline_bar.py --legend 0 --logdir $HOME/sidecar/nsdi --execute [-t 1]
```

## Figure 6

```
python figures/fig6_fairness.py --legend 0 --logdir $HOME/sidecar/nsdi --execute [-t 1]
```

## Table 6

```
cargo b --release --example benchmark_encode_multi --features benchmark,cycles
sudo -E python3 mininet/benchmark_encode.py --length 25 single -n 1 --tput 50000
sudo -E python3 mininet/benchmark_encode.py --length 1468 single -n 1 --tput 50000
```

## Figure 7

## Figure 8