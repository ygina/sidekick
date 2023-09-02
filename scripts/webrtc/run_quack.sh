#!/bin/bash
if [ $# -ne 1 ]; then
	echo "USAGE: $0 [TIMEOUT]"
	exit 1
fi

export RUST_LOG=info
$HOME/sidecar/target/release/webrtc_client --server-addr 34.221.237.169:5201 --timeout $1 \
    --quack-port 5103 --quack-style power_sum --threshold 8

