#!/bin/bash
set -e

execute=$1
python3 baseline_bar.py -t 10 $execute
python3 data_size_vs_total_time.py --mean $execute -t 10
python3 loss_vs_tput.py --max-x 800 -n 20M -t 5 $execute
python3 time_vs_cwnd.py --max-x 30 --quic-n 40M --quack-n 40M --tcp 70 --loss 0 $execute
python3 time_vs_cwnd.py --max-x 30 --quic-n 20M --quack-n 70M --tcp 70 --loss 5 $execute
python3 multiflow.py -n 15M $execute loss5p
python3 multiflow.py -n 30M $execute loss0p

