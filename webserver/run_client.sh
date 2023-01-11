#!/bin/bash
set -e

if [ "$#" -lt 2 ]; then
    echo -e "Usage:   $0 [n-bytes] [1|2|3] [trials (default: 1)] [cubic|reno]? [ip:port (default: 10.0.1.10:443)]?"
    echo -e "Example: $0 1M 3 1 reno 127.0.0.1:443"
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
    trials=1
elif [ $3 -lt 1 ]; then
    echo -e "Must run at least 1 trial"
    exit 1
else
    trials=$3
fi

if [ -z "$4" ]; then
    quiche_cc=""
else
    quiche_cc="--quiche-cc $4"
fi

# Parse the target address
if [ -z "$5" ]; then
    addr="10.0.1.10:443"
else
    addr=$5
fi

# Write the given number of bytes from /dev/urandom to a temporary file
file=$(mktemp)
cmd="head -c $1 /dev/urandom"
echo "$cmd > $file"
$cmd > $file

# Run $trials trials
echo "Data Size: $1"
echo "HTTP: $http"
if [ $trials -eq 1 ]; then
    fmt='\n\n      time_connect:  %{time_connect}s\n   time_appconnect:  %{time_appconnect}s\ntime_starttransfer:  %{time_starttransfer}s\n                   ----------\n        time_total:  %{time_total}s\n'
    cmd="curl-exp -v $http --insecure $quiche_cc --data-binary @$file https://$addr/ -w \"$fmt\""
    eval $cmd
else
    fmt='%{time_connect}\t%{time_appconnect}\t%{time_starttransfer}\t\t%{time_total}\n'
    cmd="timeout 1m curl-exp $http --insecure $quiche_cc --data-binary @$file https://$addr/ -w \"$fmt\" -o /dev/null 2>/dev/null"
    echo $cmd
    echo -e "\ntime_connect\ttime_appconnect\ttime_starttransfer\ttime_total"
    for i in $(seq 1 1 $trials); do
        eval $cmd
    done
fi

echo
rm $file

