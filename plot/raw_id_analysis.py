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
GRANULARITY = 1

def to_key(x):
    return int(x / GRANULARITY) * GRANULARITY

def collect(min_x, max_x, old_xs, old_ys):
    # Collect by second granularity
    def empty():
        return []
    xs_dict = defaultdict(empty)
    for i in range(len(old_xs)):
        x = old_xs[i]
        x = to_key(x)
        y = old_ys[i]
        xs_dict[x].append(y)
    ys = [xs_dict[x] for x in range(min_x, max_x, GRANULARITY)]
    return ys

def parse_one_file(filename):
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
        x = 1.0 * int(m[0]) + int(m[1]) / 1_000_000_000.
        y = int(m[2])
        xs.append(x)
        ys.append(y)

    return (xs, ys)

def check_subset(r1, h2):
    try:
        currset = []
        for i in range(len(r1)):
            currset += h2[i]
            for identifier in r1[i]:
                currset.remove(identifier)
        print('subset test passed :)')
        return True
    except ValueError:
        return False

def get_diff_ys(r1, h2):
    ys = []
    r1_total = 0
    h2_total = 0
    for i in range(len(r1)):
        r1_total += len(r1[i])
        h2_total += len(h2[i])
        ys.append(h2_total - r1_total)
    return ys

def parse_data(r1_filename, h2_filename):
    (r1_xs, r1_ys) = parse_one_file(r1_filename)
    (h2_xs, h2_ys) = parse_one_file(h2_filename)
    min_x = to_key(min(min(r1_xs), min(h2_xs)))
    max_x = to_key(max(max(r1_xs), max(h2_xs))) + GRANULARITY
    r1 = collect(min_x, max_x, r1_xs, r1_ys)
    h2 = collect(min_x, max_x, h2_xs, h2_ys)
    xs = [x for x in range(0, max_x - min_x, GRANULARITY)]
    assert check_subset(r1, h2)
    diff = get_diff_ys(r1, h2)

    ys = {}
    ys['r1'] = [len(y) for y in r1]
    ys['h2'] = [len(y) for y in h2]
    ys['diff'] = diff
    return (xs, ys)

def plot_graph(xs, ys, data_size):
    for (i, key) in enumerate(['r1', 'h2', 'diff']):
        plt.plot(xs, ys[key], marker=MARKERS[i], label=key)
    plt.xlabel('Time (s)')
    plt.ylabel('Num Packets')
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.4), ncol=3)
    pdf = 'rawid_{}_{}.pdf'.format(data_size, GRANULARITY)
    plt.title(pdf)
    print(pdf)
    save_pdf(pdf)
    plt.clf()

def run(args):
    r1_filename = f'{WORKDIR}/results/raw_id/r1_{args.n}.log'
    h2_filename = f'{WORKDIR}/results/raw_id/h2_{args.n}.log'

    if not path.exists(r1_filename) or not path.exists(h2_filename):
        if not args.execute:
            print(f'ERROR: path does not exist: {r1_filename} {h2_filename}')
            exit(1)
        cmd = ['sudo', '-E', 'python3', 'mininet/net.py', '-n', args.n,
               '--loss2', '0', '-t', '1', '--benchmark', 'quic', '-s', '2ms',
               '--quack-log']
        cmd += args.args
        print(' '.join(cmd))
        p = subprocess.Popen(cmd, cwd=WORKDIR, stdout='/dev/null',
            stderr='/dev/null')
        p.wait()
        os.system(f'mv {WORKDIR}/r1.log {r1_filename}')
        os.system(f'mv {WORKDIR}/h2.log {h2_filename}')

    (xs, ys) = parse_data(r1_filename, h2_filename)
    plot_graph(xs, ys, args.n)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true')
    parser.add_argument('-n', help='data size (default: 10M)', default='10M')
    parser.add_argument('--args', action='extend', nargs='+', default=[],
        help='additional arguments to append to the mininet/net.py command if executing.')
    args = parser.parse_args()

    run(args)
