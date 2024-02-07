#!/bin/bash
SERVER_URL=https://34.221.237.169
TIMEOUT=300
QUACK_THRESHOLD=10
NEAR_DELAY_MS=1
E2E_DELAY_MS=26
RESET_THRESHOLD_MS=$(( 10*$NEAR_DELAY ))

print_usage() {
	echo "USAGE: $0 [DATA_SIZE] [base|quack]"
}

if [ $# -ne 2 ]; then
	print_usage
	exit 1
fi

if ! test -f $1; then
	echo "File does not exist, try this:"
	echo "./gen_data.sh $1"
	exit 1
fi

if [ $2 == "base" ]; then
	sidecurl --http3 --insecure --data-binary @$1 $SERVER_URL --max-time $TIMEOUT \
		-w "\ntime_total: %{time_total} (%{exitcode} %{response_code} %{size_upload} %{size_download}: (%{errormsg}))\n"
elif [ $2 == "quack" ]; then
	sidecurl --http3 --insecure --data-binary @$1 $SERVER_URL --max-time $TIMEOUT \
	    -w '\ntime_total: %{time_total} (%{exitcode} %{response_code} %{size_upload} %{size_download}: (%{errormsg}))\n' \
	    --sidekick $QUACK_THRESHOLD --quack-style power_sum \
	    --enable-reset 1 --sidekick-mtu \
	    --min-ack-delay 0 --max-ack-delay 25 \
	    --near-delay $NEAR_DELAY_MS --e2e-delay $E2E_DELAY_MS \
	    --reset-threshold $RESET_THRESHOLD_MS
else
	print_usage
	exit 1
fi
