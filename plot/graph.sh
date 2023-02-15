#!/bin/bash
set -e

trials=$1
execute=$2
python3 baseline_bar.py -t $1 $execute
python3 data_size_vs_total_time.py --mean --median -t $1 $execute
python3 loss_vs_tput.py --max-x 800 -n 25M -t $1 $execute
#python3 time_vs_cwnd.py --max-x 30 --quic-n 35M --quack-n 35M --tcp 35 --loss 0 $execute
#python3 time_vs_cwnd.py --max-x 30 --quic-n 5M --quack-n 35M --tcp 35 --loss 1 $execute
python3 multiflow.py -n 60M --max-x 60 $execute loss1p
python3 multiflow.py -n 60M --max-x 60 $execute loss0p
#python3 bit_widths.py
