#!/bin/bash
sudo RUST_LOG=info $HOME/sidecar/target/release/sender -i wlp1s0 -t 8 --target-addr 10.42.0.178:5103 --frequency-pkts 2 --my-addr 10.42.0.1

