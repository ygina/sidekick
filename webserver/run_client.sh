#!/bin/bash
set -e

if [ "$#" -lt 2 ]; then
    echo -e "Usage:   $0 [n-bytes] [1|2|3] [trials]? [stdout]? [stderr]? [cubic|reno]?"
    echo -e "Example: $0 1M 3 1"
    exit 1
fi

# Parse the HTTP version
if [ $2 -eq 1 ]; then
    http="--http1.1"
elif [ $2 -eq 2 ]; then
    http="--http2"
elif [ $2 -eq 3 ]; then
    http="--http3"
else
    echo -e "HTTP version must be '1', '2', or '3'"
    exit 1
fi
echo $http

# Parse the number of trials to run
if [ -z "$3" ]; then
    trials=0
elif [ $3 -lt 1 ]; then
    echo -e "Must run at least 1 trial"
    exit 1
else
    trials=$3
fi

if [ -z "$4" ]; then
    stdout_file="/dev/null"
else
    stdout_file=$4
fi

if [ -z "$5" ]; then
    stderr_file="/dev/null"
else
    stderr_file=$5
fi

if [ -z "$6" ]; then
    quiche_cc=""
else
    quiche_cc="--quiche-cc $6"
fi

# The target address
addr="10.0.1.10:443"

# Write the given number of bytes from /dev/urandom to a temporary file
file=$(mktemp)
cmd="head -c $1 /dev/urandom"
echo "$cmd > $file"
$cmd > $file

# Run $trials trials
echo "Data Size: $1"
echo "HTTP: $http"
if [ $trials -eq 0 ]; then
    fmt='\n\n      time_connect:  %{time_connect}s\n   time_appconnect:  %{time_appconnect}s\ntime_starttransfer:  %{time_starttransfer}s\n                   ----------\n        time_total:  %{time_total}s\n\nexitcode: %{exitcode}\nresponse_code: %{response_code}\nsize_upload: %{size_upload}\nsize_download: %{size_download}\nerrormsg: %{errormsg}\n'
    cmd="curl-exp -v $http --insecure $quiche_cc --data-binary @$file https://$addr/ -w \"$fmt\""
    eval $cmd
else
    fmt='%{time_connect}\t%{time_appconnect}\t%{time_starttransfer}\t\t%{time_total}\t%{exitcode}\t\t%{response_code}\t\t%{size_upload}\t\t%{size_download}\t%{errormsg}\n'
    cmd="curl-exp $http --insecure $quiche_cc --data-binary @$file https://$addr/ -w \"$fmt\" -o $stdout_file 2>>$stderr_file"
    echo $cmd
    echo -e "\ntime_connect\ttime_appconnect\ttime_starttransfer\ttime_total\texitcode\tresponse_code\tsize_upload\tsize_download\terrormsg"
    for i in $(seq 1 1 $trials); do
        eval "$cmd --max-time 3000" || true
    done
fi

echo
rm $file

