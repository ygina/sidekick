#!/bin/bash
sudo RUST_LOG=info $HOME/sidecar/target/release/sender -i wlp1s0 -t 10 --target-addr 10.42.0.178:5103 --frequency-ms 30 --my-addr 10.42.0.1

