#!/bin/bash
SERVER_IP_PORT=34.221.237.169:5201
TIMEOUT=300
QUACK_THRESHOLD=10
QUACK_LISTEN_PORT=5103

print_usage() {
	echo "USAGE: $0 [base|quack]"
}

if [ $# -ne 1 ]; then
	print_usage
	exit 1
fi

if [ $2 == "base" ]; then
	$HOME/sidekick/target/release/media_client --server-addr $SERVER_IP_PORT \
		--timeout $TIMEOUT
elif [ $2 == "quack" ]; then
	$HOME/sidekick/target/release/media_client --server-addr $SERVER_IP_PORT \
		--timeout $TIMEOUT --threshold $QUACK_THRESHOLD \
		--quack-port $QUACK_LISTEN_PORT --quack-style power_sum
else
	print_usage
	exit 1
fi
