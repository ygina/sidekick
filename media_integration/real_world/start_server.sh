#!/bin/bash
if [ $# -ne 1 ]; then
	echo "USAGE: $0 [OUTPUT_FILE]"
	exit 1
fi

export RUST_LOG=debug
$HOME/sidecar/target/release/webrtc_server --port 5201 --rtt 110 --loop 2>&1 | tee -a $1

