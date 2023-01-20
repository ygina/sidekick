#!/bin/bash
if [ $# -ne 1 ]; then
	echo "USAGE: $0 [0|1]"
	echo "0 = kill sidecar process"
	echo "<positive_integer> = start quack sender at this frequency"
	exit 1
fi

if [ $1 -eq 0 ]; then
	kill $(pidof sidecar)
else
	RUST_LOG=error ./target/release/sidecar --interface r1-eth1 quack-sender --target-addr 10.0.2.10:5103 --frequency-ms $1 &
fi
