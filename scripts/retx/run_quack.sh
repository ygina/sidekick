#!/bin/bash
if [ $# -ne 1 ]; then
	echo "USAGE: $0 [DATA_SIZE]"
	exit 1
fi

sidecurl --http3 --insecure --data-binary @$1 https://34.221.237.169 --max-time 300 \
    --threshold 10 --quack-reset --sidecar-mtu --quack-style power_sum \
    -w "\ntime_total: %{time_total}\nexitcode: %{exitcode}\nresponse_code: %{response_code}\nsize_upload: %{size_upload}\nsize_download: %{size_download}\nerrormsg: %{errormsg}\n"

