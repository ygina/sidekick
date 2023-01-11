#!/bin/bash
if [ $# -ne 2 ]; then
  echo "USAGE: $0 [loss] [tcp|pep|quic]"
  exit 1
fi

loss=$1
if [ $2 == "pep" ]; then
  bm="tcp --pep"
else
  bm=$2
fi

for cc in cubic reno; do
  DIRECTORY=results/loss${loss}p/${cc}
  mkdir -p $DIRECTORY
  for data_size in $(seq 100 100 1000); do
    echo $cc ${data_size}k
    sudo python3 mininet/net.py --loss2 $loss --benchmark $bm -cc $cc -t 7 -n ${data_size}k 2> >(tee -a ${DIRECTORY}/$2.txt)
  done
done

