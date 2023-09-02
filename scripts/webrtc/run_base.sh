#!/bin/bash
if [ $# -ne 1 ]; then
	echo "USAGE: $0 [TIMEOUT]"
	exit 1
fi

$HOME/sidecar/target/release/webrtc_client --server-addr 34.221.237.169:5201 --timeout $1

