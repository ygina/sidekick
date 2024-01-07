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
        m = re.search(r'Target rate is ([\d\.]+) packets/s', line)
        if m is not None:
            target_rate = int(m.group(1))
            continue
        m = re.search(r'Combined rate \(packets/s\): ([\d\.]+)', line)
        if m is not None:
            achieved_rate = float(m.group(1))
            assert parsed.next_target_rate() * parsed.num_clients == target_rate
            parsed.add_achieved_rate(achieved_rate)
            target_rate = None
            continue
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
        """
        We use "single" and not "multi" because both involve sniffing and the
        hash table lookup, but "single" ensures we lookup the same quACK every
        time to avoid confounding the results with allocations and cache misses.
        """
        return ['sudo', '-E', 'python3', 'mininet/benchmark_encode.py',
                '--warmup', str(args.warmup), '--timeout', str(args.timeout),
                '--length', str(payload),
                '--threshold', str(threshold), 'single',
                '--num-clients', str(args.num_clients), '--tput', str(pps)]

    # While we still haven't performed NUM_BISECTIONS bisections
    while not parsed.done():
        cmd = gen_cmd(parsed.next_target_rate())
        print(' '.join(cmd))
        p = subprocess.Popen(cmd, cwd=args.workdir, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT)
        success = False
        with open(filename, 'ab') as f:
            f.write(bytes(' '.join(cmd) + '\n', 'utf-8'))
            for line in p.stdout:
                f.write(line)
                sys.stdout.buffer.write(line)
                sys.stdout.buffer.flush()
                m = re.search(r'Combined rate \(packets/s\): ([\d\.]+)', line.decode('utf-8'))
                if m is not None:
                    achieved_rate = float(m.group(1))
                    parsed.add_achieved_rate(achieved_rate)
                    success = True
        if not success:
            print('ERROR')
            break
    return parsed

if __name__ == '__main__':
    parser.add_argument('--payload', required=True, type=int, help=f'payload size')
    parser.add_argument('--threshold', '-t', type=int, default=20,
        help=f'(default: 20)')
    parser.add_argument('--warmup', default=5, type=int, help='(default: 5)')
    parser.add_argument('--timeout', default=10, type=int, help='(default: 10)')
    parser.add_argument('--num-clients', default=15, type=int,
        help='number of clients, should be number of cores minus one (default: 15)')
    parser.add_argument('--initial-rate', default=100000, type=int,
        help='initial target rate per client, in packets per second. should be '
             'larger than the highest achievable rate (default: 100000)')
    parser.add_argument('--prefix', default='', type=str,
        help='results filename prefix (default = \'\')')
    args = parser.parse_args()

    # Parse results data, and collect missing data points if specified.
    data = {}
    os.system(f'mkdir -p {args.logdir}/cpu')
    xs = []
    ys = []
    filename = f'{args.logdir}/cpu/payload{args.payload}_threshold{args.threshold}.txt'
    os.system(f'touch {filename}')
    parsed = parse_and_maybe_collect_missing_data(
        filename, args.payload, args.threshold, args)
    print(f'target_rates = {parsed.target_rates}')
    print(f'achieved_rates = {parsed.achieved_rates}')
    if parsed.done():
        print(f'max_achieved_rate = {parsed.max_achieved_rate()}')
