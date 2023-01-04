#!/bin/bash
if [ "$#" -lt 2 ]; then
    echo -e "Usage:   $0 [n-bytes] [h1|h2|h3] [ip:port (default: 10.0.1.10:443)]?"
    echo -e "Example: $0 1M h3 127.0.0.1:443"
    exit 1
fi

if [ $2 == "h1" ]; then
    http="--http1.1"
elif [ $2 == "h2" ]; then
    http="--http2"
elif [ $2 == "h3" ]; then
    http="--http3"
else
    echo -e "HTTP version must be 'h1', 'h2', or 'h3'"
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
cmd="curl $http --insecure --data-binary @$file -v https://$addr/"
echo $cmd
$cmd
echo
rm $file

