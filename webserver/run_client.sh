#!/bin/bash
set -e

if [ "$#" -lt 2 ]; then
    echo -e "Usage:   $0 [n-bytes] [1|2|3] [ip:port (default: 10.0.1.10:443)]?"
    echo -e "Example: $0 1M 3 127.0.0.1:443"
    exit 1
fi

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

if [ -z "$3" ]; then
    addr="10.0.1.10:443"
else
    addr=$3
fi

file=$(mktemp)
cmd="head -c $1 /dev/urandom"
echo "$cmd > $file"
$cmd > $file

# https://superuser.com/questions/590099/can-i-make-curl-fail-with-an-exitcode-different-than-0-if-the-http-status-code-i
fmt='\n\n      time_connect:  %{time_connect}s\n   time_appconnect:  %{time_appconnect}s\ntime_starttransfer:  %{time_starttransfer}s\n                   ----------\n        time_total:  %{time_total}s\n'
cmd="curl $http --insecure --data-binary @$file https://$addr/ -w \"$fmt\""
echo $cmd
eval $cmd
echo
rm $file

