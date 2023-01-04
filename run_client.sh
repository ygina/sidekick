#!/bin/bash
if [ "$#" -ne 2 ]; then
    echo -e "Usage:   $0 [n-bytes] [h1|h2|h3]"
    echo -e "Example: $0 1M h3"
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

file=$(mktemp)
cmd="head -c $1 /dev/urandom"
echo "$cmd > $file"
$cmd > $file

# https://superuser.com/questions/590099/can-i-make-curl-fail-with-an-exitcode-different-than-0-if-the-http-status-code-i
cmd="curl $http --insecure --data-binary @$file -v https://127.0.0.1:443/"
echo $cmd
$cmd
echo
rm $file

