#!/bin/bash
if [ $# -ne 2 ]; then
	echo "USAGE: $0 [DATA_SIZE] [MAX_TIME]"
	exit 1
fi

sidecurl --http3 --insecure --data-binary @$1 https://34.221.237.169 --max-time $2 \
    --threshold 10 --quack-reset --sidecar-mtu --quack-style power_sum \
    -w "\ntime_total: %{time_total} (%{exitcode} %{response_code} %{size_upload} %{size_download}: (%{errormsg}))\n"

