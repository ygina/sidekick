#!/bin/bash
if [ $# -ne 1 ]; then
	echo "USAGE: $0 [DATA_SIZE]"
	exit 1
fi

head -c $1 /dev/urandom > $1

