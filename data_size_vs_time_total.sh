#!/bin/bash
if [ $# -ne 5 ]; then
  echo "USAGE: $0 [loss] [tcp|pep|quic] [trials] [min] [max]"
  exit 1
fi

loss=$1
if [ $2 == "pep" ]; then
  bm="tcp --pep"
else
  bm=$2
fi
trials=$3

for cc in cubic; do
  DIRECTORY=results/loss${loss}p/${cc}
  mkdir -p $DIRECTORY
  for data_size in $(seq $4 100 $5); do
    echo $cc ${data_size}k
    sudo python3 mininet/net.py --loss2 $loss --benchmark $bm -cc $cc -t $trials -n ${data_size}k 2> >(tee -a ${DIRECTORY}/$2.txt)
  done
done

