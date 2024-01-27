#!/bin/bash
MEDIA_LISTEN_PORT=5201
E2E_RTT_MS=110

if [ $# -ne 1 ]; then
	echo "USAGE: $0 [OUTPUT_FILE]"
	exit 1
fi

RUST_LOG=debug $HOME/sidecar/target/release/media_server \
	--port $MEDIA_LISTEN_PORT --rtt $E2E_RTT_MS --loop 2>&1 | tee -a $1
