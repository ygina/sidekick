import argparse
import subprocess
import os
import re
import sys
import os.path
import statistics
from collections import defaultdict
from os import path
from common import *

WORKDIR = os.environ['HOME'] + '/sidecar'
GRANULARITY_MS = None
SEND = 0
RECV = 1
LOST_QUACK = 2
LOST_E2E = 3
CWND = 4

def to_key(x):
    return int(x / GRANULARITY) * GRANULARITY

def collect_data(data, filter_action):
    def empty():
        return []
    xs_dict = defaultdict(empty)
    min_x = to_key(data[0][0])
    max_x = to_key(data[-1][0]) + GRANULARITY

    for (x, action, identifier) in data:
        if action != filter_action:
            continue
        x = to_key(x)
        xs_dict[x].append(identifier)
    ys = [xs_dict[x] for x in range(min_x, max_x, GRANULARITY)]
    return ys

def parse_quack(filename):
    with open(filename) as f:
        lines = f.read().split('\n')
    xs = []
    ys = []

    for line in lines:
        line = line.strip()
        r = r'^quack Instant \{ tv_sec: (\d+), tv_nsec: (\d+) \} (\d+)$'
        m = re.search(r, line)
        if m is None:
            continue
        m = m.groups()
        x = 1000.0 * int(m[0]) + int(m[1]) / 1_000_000.
        y = int(m[2])
        xs.append(x)
        ys.append(y)

    return (xs, ys)

def parse_lost(filename, function):
    with open(filename) as f:
        lines = f.read().split('\n')
    xs = []
    ys = []

    for line in lines:
        line = line.strip()

        r = r'^lost Instant \{ tv_sec: (\d+), tv_nsec: (\d+) \} (\d+) \((.*)\)$'
        m = re.search(r, line)
        if m is None:
            continue
        m = m.groups()
        if m[3] != function:
            continue
        x = 1000.0 * int(m[0]) + int(m[1]) / 1_000_000.
        y = int(m[2])
        xs.append(x)
        ys.append(y)

    return (xs, ys)

def parse_cwnd(filename):
    with open(filename) as f:
        lines = f.read().split('\n')
    xs = []
    ys = []

    for line in lines:
        line = line.strip()

        r = r'^cwnd (\d+) Instant \{ tv_sec: (\d+), tv_nsec: (\d+) \} \((.*)\)$'
        m = re.search(r, line)
        if m is None:
            continue
        m = m.groups()
        x = 1000.0 * int(m[1]) + int(m[2]) / 1_000_000.
        y = int(m[0]) / 1460.
        xs.append(x)
        ys.append(y)

    return (xs, ys)

def combine_data(r1, h2, lost_quack, lost_e2e, cwnd):
    data = []
    for i in range(len(h2[0])):
        data.append((h2[0][i], SEND, h2[1][i]))
    for i in range(len(r1[0])):
        data.append((r1[0][i], RECV, r1[1][i]))
    for i in range(len(lost_quack[0])):
        data.append((lost_quack[0][i], LOST_QUACK, lost_quack[1][i]))
    for i in range(len(lost_e2e[0])):
        data.append((lost_e2e[0][i], LOST_E2E, lost_e2e[1][i]))
    for i in range(len(cwnd[0])):
        data.append((cwnd[0][i], CWND, cwnd[1][i]))
    data.sort()
    return data

def check_subset(data):
    currset = []
    for (_time, action, identifier) in data:
        if action == SEND:
            currset.append(identifier)
        elif action == RECV or action == LOST_QUACK:
            index = currset.index(identifier)
            if action == LOST_QUACK:
                assert index == 0
            currset.remove(identifier)
        elif action == CWND or action == LOST_E2E:
            continue
    print('subset test passed')

def parse_data(r1_filename, h2_filename):
    # Parse raw data and check subset properties
    r1 = parse_quack(r1_filename)
    h2 = parse_quack(h2_filename)
    lost_quack = parse_lost(h2_filename, 'on_quack_received')
    lost_e2e = parse_lost(h2_filename, 'detect_lost_packets')
    cwnd = parse_cwnd(h2_filename)
    data = combine_data(r1, h2, lost_quack, lost_e2e, cwnd)
    check_subset(data)

    # Collect data for plotting
    min_x = to_key(data[0][0])
    max_x = to_key(data[-1][0]) + GRANULARITY
    r1 = collect_data(data, RECV)
    h2 = collect_data(data, SEND)
    lost_quack = collect_data(data, LOST_QUACK)
    lost_e2e = collect_data(data, LOST_E2E)
    cwnd = collect_data(data, CWND)
    xs = [x / 1000. for x in range(0, max_x - min_x, GRANULARITY)]

    ys = {}
    ys['r1'] = [len(y) for y in r1]
    ys['h2'] = [len(y) for y in h2]
    ys['h2-r1'] = [(ys['h2'][i]-ys['r1'][i]) for i in range(len(ys['r1']))]
    count = 0
    ys['diff'] = []
    for (i, d) in enumerate(ys['h2-r1']):
        count += d - len(lost_quack[i])
        ys['diff'].append(count)

    ys['lost_quack'] = [len(y) for y in lost_quack]
    ys['lost_e2e'] = [len(y) for y in lost_e2e]
    ys['cwnd'] = []
    for y in cwnd:
        if len(y) > 0:
            ys['cwnd'].append(statistics.mean(y))
        else:
            ys['cwnd'].append(ys['cwnd'][-1])
    return (xs, ys)

def plot_graph(xs, ys, data_size, loss, threshold):
    # for (i, key) in enumerate(['r1', 'h2', 'diff', 'lost_quack', 'lost_e2e', 'h2-r1']):
    for (i, key) in enumerate(['r1', 'h2', 'diff', 'lost_quack', 'h2-r1']):
        plt.plot(xs, ys[key], label=key)
    plt.plot(xs, ys['cwnd'], label='cwnd')
    plt.xlabel('Time (s)')
    plt.ylabel('Num Packets')
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.7), ncol=3)
    pdf = f'rawid_{data_size}_loss{loss}p_thresh{threshold}.pdf'
    plt.title(pdf)
    print(pdf)
    save_pdf(pdf)
    plt.clf()

def run(args):
    r1_filename = f'{WORKDIR}/results/raw_id/r1_{args.n}_loss{args.loss}p_thresh{args.t}.log'
    h2_filename = f'{WORKDIR}/results/raw_id/h2_{args.n}_loss{args.loss}p_thresh{args.t}.log'

    if not path.exists(r1_filename) or not path.exists(h2_filename) or args.f:
        if not args.execute:
            print(f'ERROR: path does not exist: {r1_filename} {h2_filename}')
            exit(1)
        cmd = ['sudo', '-E', 'python3', 'mininet/net.py', '-n', args.n,
               '--loss2', args.loss, '-t', '1', '--benchmark', 'quic',
               '-s', '2ms', '--quack-log', '--threshold', args.t]
        cmd += args.args
        print(' '.join(cmd))
        p = subprocess.Popen(cmd, cwd=WORKDIR)
        p.wait()
        os.system(f'mv {WORKDIR}/r1.log {r1_filename}')
        os.system(f'mv {WORKDIR}/h2.log {h2_filename}')

    (xs, ys) = parse_data(r1_filename, h2_filename)
    plot_graph(xs, ys, args.n, args.loss, args.t)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true')
    parser.add_argument('-f', help='force execute', action='store_true')
    parser.add_argument('-n', help='data size (default: 10M)', default='10M')
    parser.add_argument('-t', help='quack threshold (default: 500)', default='500')
    parser.add_argument('--loss', help='loss (default: 0)', default='0')
    parser.add_argument('--args', action='extend', nargs='+', default=[],
        help='additional arguments to append to the mininet/net.py command if executing.')
    parser.add_argument('-g', help='time granularity in ms (default: 1000)',
        type=int, default=1000)
    args = parser.parse_args()
    GRANULARITY = args.g

    run(args)
