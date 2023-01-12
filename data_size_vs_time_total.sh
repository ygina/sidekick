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
  SUFFIX=loss${loss}p/${cc}
  mkdir -p results/$SUFFIX
  # mkdir -p stdout/$SUFFIX
  # mkdir -p stderr/$SUFFIX
  RESULTS_FILE=results/$SUFFIX/$2.txt
  STDOUT_FILE=/dev/null
  STDERR_FILE=/dev/null
  for data_size in $(seq $4 100 $5); do
    echo $cc ${data_size}k
    sudo python3 mininet/net.py --loss2 $loss --benchmark $bm -cc $cc \
      --stdout ${STDOUT_FILE} --stderr ${STDERR_FILE} \
      -t $trials -n ${data_size}k 2> >(tee -a ${RESULTS_FILE})
  done
done

