import argparse
import subprocess
import os
import re
import sys
import os.path
import statistics
import math
from os import path
from collections import defaultdict
from common import *

NUM_BISECTIONS = 9
THRESHOLDS = [x for x in range(0, 380, 20)]
WORKDIR = os.environ['HOME'] + '/sidecar'

class ParsedFile:
    def __init__(self, payload, threshold, args):
        self.payload = int(payload)
        self.threshold = int(threshold)
        self.num_clients = args.num_clients
        # For binary search
        self.hi = args.initial_rate
        self.lo = 0
        # List of target and achieved rates
        self.target_rates = []
        self.achieved_rates = []

    def add_achieved_rate(self, achieved_rate):
        mid = self.next_target_rate()
        target_rate = mid * self.num_clients
        self.target_rates.append(target_rate)
        self.achieved_rates.append(achieved_rate)
        if len(self.target_rates) > 1:
            if math.isclose(achieved_rate, target_rate, rel_tol=0.01):
                # Rates are close enough, increase the rate.
                self.lo = mid
            else:
                self.hi = mid

    def next_target_rate(self):
        if len(self.target_rates) == 0:
            return self.hi
        else:
            return int((self.lo + self.hi) / 2)

    def max_achieved_rate(self):
        return max(self.achieved_rates)

    def done(self):
        return len(self.target_rates) == NUM_BISECTIONS

def parse_data(filename, payload, threshold, args):
    parsed = ParsedFile(payload, threshold, args)

    achieved_rate = None
    target_rate = None

    with open(filename) as f:
        lines = f.read().split('\n')
    for line in lines:
        line = line.strip()

        # Parse the target and achieved rates
        m = re.search(r'Rate \(packets/s\): ([\d\.]+)', line)
        if m is not None:
            achieved_rate = float(m.group(1))
            continue
        m = re.search(r'Target combined rate \(packets/s\): ([\d\.]+)', line)
        if m is not None:
            target_rate = float(m.group(1))
            assert parsed.next_target_rate() * parsed.num_clients == target_rate
            parsed.add_achieved_rate(achieved_rate)
            target_rate = None
            achieved_rate = None
    # print([x for x in zip(parsed.target_rates, parsed.achieved_rates)])
    return parsed

def parse_and_maybe_collect_missing_data(filename, payload, threshold, args):
    print(filename)
    parsed = parse_data(filename, payload, threshold, args)
    if parsed.done():
        return parsed
    if not args.execute:
        print('INCOMPLETE')
        return parsed

    def gen_cmd(pps):
        return ['sudo', '-E', 'python3', 'mininet/benchmark_encode.py',
                '--warmup', str(args.warmup), '--timeout', str(args.timeout),
                '--length', str(payload),
                '--threshold', str(threshold), 'single',
                '--num-clients', str(args.num_clients), '--tput', str(pps)]

    # While we still haven't performed NUM_BISECTIONS bisections
    while not parsed.done():
        cmd = gen_cmd(parsed.next_target_rate())
        print(' '.join(cmd))
        p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT)
        success = False
        with open(filename, 'ab') as f:
            f.write(bytes(' '.join(cmd) + '\n', 'utf-8'))
            for line in p.stdout:
                f.write(line)
                sys.stdout.buffer.write(line)
                sys.stdout.buffer.flush()
                m = re.search(r'Rate \(packets/s\): ([\d\.]+)', line.decode('utf-8'))
                if m is not None:
                    achieved_rate = float(m.group(1))
                    parsed.add_achieved_rate(achieved_rate)
                    success = True
        if not success:
            print('ERROR')
            break
    return parsed

def plot_graph(data, payloads, throughput=False):
    HEADERS_SIZE = 14 + 20 + 8
    plt.figure(figsize=(8, 6))
    for (i, payload) in enumerate(payloads):
        (xs, ys) = data[payload]
        if throughput:
            ys = [(HEADERS_SIZE + payload) * pps * 8 / 1000000000 for pps in ys]
        plt.plot(xs, ys, marker=MARKERS[i], label=f'payload{payload}')
    plt.xlabel('Threshold')
    if throughput:
        plt.ylabel('Max Rate Achieved (Gbit/s)')
    else:
        plt.ylabel('Max Rate Achieved (pps)')
    plt.xlim(0)
    plt.ylim(0)
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.3), ncol=2)
    if throughput:
        pdf = 'cpu_single_gbits.pdf'
    else:
        pdf = 'cpu_single_pps.pdf'
    plt.title(pdf)
    if pdf:
        save_pdf(f'{WORKDIR}/plot/graphs/{pdf}')

def plot_cpu_cycles_bar_graph(pdf='cpu_cycles.pdf'):
    data = []
    data.append([27424, 27193])
    data.append([17808, 18012])
    data.append([44, 38])
    data.append([441, 449])
    data.append([234, 255])
    labels = ['Sniff Packet', 'Hash Key', 'Parse ID', 'Encode ID', 'Other']

    def offsets(index):
        offset25 = sum([x[0] for x in data[:index]])
        offset1468 = sum([x[1] for x in data[:index]])
        return [offset25, offset1468]

    plt.figure(figsize=(8, 4))
    xticks = [0, 1]
    plt.barh(xticks, data[0], left=offsets(0), label=labels[0])
    plt.barh(xticks, data[1], left=offsets(1), label=labels[1])
    plt.barh(xticks, data[2], left=offsets(2), label=labels[2])
    plt.barh(xticks, data[3], left=offsets(3), label=labels[3])
    plt.barh(xticks, data[4], left=offsets(4), label=labels[4])

    plt.legend(loc='right', bbox_to_anchor=(1.6, 0.5), ncol=1)
    plt.title(pdf)
    if pdf:
        save_pdf(f'{WORKDIR}/plot/graphs/{pdf}')

if __name__ == '__main__':
    DEFAULT_PAYLOADS = [25, 1468]

    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true',
        help='whether to execute benchmarks to collect missing data points')
    parser.add_argument('--plot', action='store_true',
        help='plot the threshold vs throughput graphs')
    parser.add_argument('--bar-graph', action='store_true',
        help='plot the cpu cycles horizontal stacked bar graphs')
    parser.add_argument('--payloads', action='extend', nargs='+', type=int,
        help=f'payload sizes. (default: {DEFAULT_PAYLOADS})', default=[])
    parser.add_argument('--threshold', '-t', action='extend', nargs='+', type=int,
        help=f'thresholds. (default: {THRESHOLDS})', default=[])
    parser.add_argument('--warmup', default=5, type=int, help='(default: 5)')
    parser.add_argument('--timeout', default=10, type=int, help='(default: 10)')
    parser.add_argument('--max-x', default=360, type=int,
        help='maximum threshold to plot (default: 360)')
    parser.add_argument('--num-clients', default=15, type=int,
        help='number of clients (default: 15)')
    parser.add_argument('--initial-rate', default=200000, type=int,
        help='initial target rate per client, in packets per second. should be '
             'larger than the highest achievable rate (default: 200000)')
    parser.add_argument('--prefix', default='', type=str,
        help='results filename prefix (default = \'\')')
    args = parser.parse_args()

    # Parse results data, and collect missing data points if specified.
    data = {}
    payloads = DEFAULT_PAYLOADS if len(args.payloads) == 0 else args.payloads
    thresholds = THRESHOLDS if len(args.threshold) == 0 else args.threshold
    for payload in payloads:
        path = f'{WORKDIR}/results/cpu/payload{payload}'
        os.system(f'mkdir -p {path}')
        xs = []
        ys = []
        for threshold in thresholds:
            if threshold > args.max_x:
                continue
            filename = f'{path}/{args.prefix}threshold{threshold}.txt'
            os.system(f'touch {filename}')
            parsed = parse_and_maybe_collect_missing_data(
                filename, payload, threshold, args)
            if parsed.done():
                xs.append(threshold)
                ys.append(parsed.max_achieved_rate())
        data[payload] = (xs, ys)
    print(data)

    # Plot data.
    if args.plot:
        plot_graph(data, payloads, throughput=False)
        plot_graph(data, payloads, throughput=True)
    if args.bar_graph:
        plot_cpu_cycles_bar_graph()
