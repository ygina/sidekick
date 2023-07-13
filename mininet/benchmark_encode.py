import argparse
import sys
import time
from network import *
from mininet.log import setLogLevel

if __name__ == '__main__':
    setLogLevel('info')

    parser = argparse.ArgumentParser(prog='Benchmark Encoder')

    ############################################################################
    # Network Configurations
    parser.add_argument('--tput', type=int, default=1000, metavar='PPS',
        help='Target load generator throughput (packets/s) (default: 1000)')
    parser.add_argument('--warmup', '-w', type=int, default=3, metavar='S',
        help='Warmup time, in seconds (default: 3)')
    parser.add_argument('--timeout', '-t', type=int, default=5, metavar='S',
        help='Timeout, in seconds (default: 5)')
    parser.add_argument('--frequency', type=int, default=0,
        help='Quack frequency in ms (default: 0)')
    parser.add_argument('--threshold', type=int, default=20, metavar='PACKETS',
        help='Quack threshold (default: 20)')
    parser.add_argument('--bw', type=int, default=10000, metavar='MBITS',
        help='Link bandwidth (in Mbit/s) (default: 10000)')

    args = parser.parse_args()
    net = SidecarNetwork(delay1=0, delay2=0, loss1=0, loss2=0, bw1=args.bw,
        bw2=args.bw, qdisc='codel')

    """
    1. Set up the network
    2. Start the load generator at the given rate on h2.
        * The load generator collects statistics from the first packet sent
          after the warmup time.
    3. Start the quack sender listening on r1.
        * The quack sender collects statistics from the first packet sniffed.
    4. Wait <timeout> seconds.
    5. Stop the quack sender and load generator. Print statistics.
        * Quack sender: tput (packets/s); latency median, mean, stdev, min,
          max (ns/packet)
        * Load generator: target tput (packets/s); tput (packets/s)
    """
    print(net.h1.popen(f'iperf3 -s -f m'.split(' ')))
    # load_generator = net.h2.popen(f'./target/release/load_generator --warmup {args.warmup} --tput {args.tput}'.split(' '))
    print(net.h2.popen(f'iperf3 -c 0.0.0.0 --udp --time {args.warmup + args.timeout} --format m --congestion cubic -b {int(args.tput * 1460 / 1000)}K'.split(' ')))
    time.sleep(args.warmup)
    p2 = net.r1.popen(f'./target/release/benchmark_encode --threshold {args.threshold} --frequency {args.frequency}'.split(' '))
    time.sleep(args.timeout)
    p2.terminate()
    for line in p2.stdout:
        if b'DONE' in line:
            sys.stdout.buffer.flush()
            break
        sys.stdout.buffer.write(line)
    net.stop()
