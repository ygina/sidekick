#!/bin/bash
NET_IFACE=wlp1s0
QUACK_THRESHOLD=10
FREQUENCY_MS=30
CLIENT_QUACK_IP_PORT=10.42.0.178:5103
MY_PUBLIC_IP=10.42.0.1

sudo RUST_LOG=info $HOME/sidecar/target/release/sender -i $NET_IFACE \
	-t $QUACK_THRESHOLD --frequency-ms $FREQUENCY_MS \
	--target-addr $CLIENT_QUACK_IP_PORT --my-addr $MY_PUBLIC_IP
