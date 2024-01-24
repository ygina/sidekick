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
export SIDECAR_HOME=$HOME/sidecar
export QUICHE_HOME=$HOME/sidecar/http3_integration/quiche
```

## Table 3

```
cargo b --release --examples --all-features
export RUST_LOG=warn
./target/release/examples/benchmark_construct strawman1a
./target/release/examples/benchmark_construct strawman1b
./target/release/examples/benchmark_construct strawman2
./target/release/examples/benchmark_construct power-sum
./target/release/examples/benchmark_decode strawman2 --trials 1
./target/release/examples/benchmark_decode power-sum
```

## Figure 2

```
python figures/fig2_microbenchmarks.py --logdir $SIDECAR_HOME/nsdi --execute
```

## Figure 4

### Figure 4a

```
python figures/fig4a_pep_emulation.py --marquee --logdir $SIDECAR_HOME/nsdi --execute [-t 1]
```

### Figure 4b

```
python figures/fig4b_low_latency_media.py --logdir $SIDECAR_HOME/nsdi --execute [-t 1]
```

### Figure 4c

```
python figures/fig4c_ack_reduction.py --marquee --logdir $SIDECAR_HOME/nsdi --execute
```

## Figure 5

```
python figures/fig5_baseline_bar.py --legend 0 --logdir $SIDECAR_HOME/nsdi --execute [-t 1]
```

## Figure 6

```
python figures/fig6_fairness.py --legend 0 --logdir $SIDECAR_HOME/nsdi --execute [-t 1]
```

## Section 6.4 Proxy CPU Overheads

### Table 6

```
cargo b --release --example benchmark_encode_multi --features benchmark,cycles
sudo -E python3 mininet/benchmark_encode.py --length 25 single -n 1 --tput 50000
sudo -E python3 mininet/benchmark_encode.py --length 1468 single -n 1 --tput 50000
```

#### Max Throughput

Note you will need at least 16 cores to achieve enough load, such as an
m4.4xlarge AWS instance.

```
cargo b --release --example benchmark_encode_multi --features benchmark
python figures/cpu.py --payload 25 --num-clients 15 --execute
python figures/cpu.py --payload 1468 --num-clients 15 --execute
```

## Section 6.5 Link Overheads

For each experiment in Figure 7, collect the `time_total`, and the `tx_packets`
and `tx_bytes`
from DS->proxy, DS<-proxy, and proxy<-DR (the 1st, 2nd, and 4th rows). Calculate
the goodput by dividing the data size 10 MBytes from the total time.

The first row is the QUIC E2E baseline. The following rows in the table in the
paper are calculated relative to the first row.

### Figure 7a

```
cd $QUICHE_HOME && make sidecar && cd $SIDECAR_HOME
sudo -E python3 mininet/main.py -t 1 --loss2 1 --delay1 25 --delay2 1 --bw1 10 --bw2 100 -n 10M --print-statistics quic
cd $QUICHE_HOME && make strawman_a && cd $SIDECAR_HOME
sudo -E python3 mininet/main.py -t 1 --loss2 1 --delay1 25 --delay2 1 --bw1 10 --bw2 100 -n 10M --frequency 30ms --threshold 10 --print-statistics quack --style strawman_a
cd $QUICHE_HOME && make strawman_b && cd $SIDECAR_HOME
sudo -E python3 mininet/main.py -t 1 --loss2 1 --delay1 25 --delay2 1 --bw1 10 --bw2 100 -n 10M --frequency 30ms --threshold 10 --print-statistics quack --style strawman_b
cd $QUICHE_HOME && make strawman_c && cd $SIDECAR_HOME
sudo -E python3 mininet/main.py -t 1 --loss2 1 --delay1 25 --delay2 1 --bw1 10 --bw2 100 -n 10M --frequency 30ms --threshold 10 --print-statistics quack --style strawman_c
cd $QUICHE_HOME && make sidecar && cd $SIDECAR_HOME
sudo -E python3 mininet/main.py -t 1 --loss2 1 --delay1 25 --delay2 1 --bw1 10 --bw2 100 -n 10M --frequency 30ms --threshold 10 --print-statistics quack --style power_sum
```

### Figure 7b

```
cd $QUICHE_HOME && make sidecar && cd $SIDECAR_HOME
sudo -E python3 mininet/main.py -t 1 --loss2 0 --delay1 1 --delay2 25 --bw1 100 --bw2 10 -n 10M --min-ack-delay 10 --print-statistics quic
cd $QUICHE_HOME && make strawman_a && cd $SIDECAR_HOME
sudo -E python3 mininet/main.py -t 1 --loss2 0 --delay1 1 --delay2 25 --bw1 100 --bw2 10 -n 10M --frequency 10ms --threshold 40 --min-ack-delay 500 --print-statistics --timeout 20 quack --style strawman_a
cd $QUICHE_HOME && make strawman_b && cd $SIDECAR_HOME
sudo -E python3 mininet/main.py -t 1 --loss2 0 --delay1 1 --delay2 25 --bw1 100 --bw2 10 -n 10M --frequency 10ms --threshold 40 --min-ack-delay 500 --print-statistics --timeout 20 quack --style strawman_b
cd $QUICHE_HOME && make strawman_c && cd $SIDECAR_HOME
sudo -E python3 mininet/main.py -t 1 --loss2 0 --delay1 1 --delay2 25 --bw1 100 --bw2 10 -n 10M --frequency 10ms --threshold 40 --min-ack-delay 500 --print-statistics --timeout 20 quack --style strawman_c
cd $QUICHE_HOME && make sidecar && cd $SIDECAR_HOME
sudo -E python3 mininet/main.py -t 1 --loss2 0 --delay1 1 --delay2 25 --bw1 100 --bw2 10 -n 10M --frequency 10ms --threshold 40 --min-ack-delay 500 --print-statistics --timeout 20 quack --style power_sum

```

### QuACK vs ACK Cycles

```
cd $QUICHE_HOME && make cycles && cd $SIDECAR_HOME
sudo -E python3 mininet/main.py -t 1 --loss2 1 --delay1 25 --delay2 1 --bw1 10 --bw2 100 -n 10M --frequency 30ms --threshold 10 quack
```

## Figure 8