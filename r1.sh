#!/bin/bash
./target/release/sidecar --interface r1-eth1 quack-sender --target-addr 10.0.2.10:53535 --frequency-ms $1
