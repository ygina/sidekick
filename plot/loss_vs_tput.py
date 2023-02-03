import argparse
import subprocess
import os
import re
import sys
import os.path
import statistics
from os import path
from collections import defaultdict
from common import *

KEYS = ['tcp', 'quic']
TARGET_XS = [x for x in range(20)]
WORKDIR = os.environ['HOME'] + '/sidecar'

def empty_list():
    return []

def collect_ys(ys, n):
    assert n[-1] == 'M'
    n_megabyte = int(n[:-1]) * 1.0
    return [n_megabyte / statistics.mean(ys)]

def parse_data(filename, trials, max_x, data_key='time_total'):
    loss = None
    key_index = None
    exitcode_index = None
    data = defaultdict(empty_list)

    with open(filename) as f:
        lines = f.read().split('\n')
    for line in lines:
        line = line.strip()

        # Get the current loss percentage in tenths of a percent
        m = re.search(r'.*1ms delay (\S+)% loss.*', line)
        if m is not None:
            loss = round(float(m.group(1)) * 10.0)
            continue

        # Figure out which index to parse the total time and exitcode
        if data_key in line:
            keys = line.split()
            for i in range(len(keys)):
                if keys[i] == data_key:
                    key_index = i
                elif keys[i] == 'exitcode':
                    exitcode_index = i
            continue
        if key_index is None:
            continue

        # Either we're done with this loss percentage or read another data point
        if line == '' or '***' in line or '/tmp' in line or 'No' in line or \
            'factor' in line or 'unaccounted' in line:
            loss = None
            key_index = None
            exitcode_index = None
        else:
            line = line.split()
            if exitcode_index is not None and int(line[exitcode_index]) != 0:
                continue
            data[loss].append(float(line[key_index]))

    xs = [x for x in filter(lambda x: x <= max_x, TARGET_XS)]
    xs.sort()
    ys = [data[x][:min(len(data[x]), trials)] for x in xs]
    return (xs, ys)

def maybe_collect_missing_data(filename, key, args):
    (xs, ys) = parse_data(filename, args.trials, args.max_x)

    missing_losses = []
    for i in range(len(xs)):
        missing = max(0, args.trials - len(ys[i]))
        loss = f'{i*0.1:.1f}'
        if missing == args.trials:
            missing_losses.append(loss)
        elif missing > 0:
            print(f'{i*0.1:.1f}% {len(ys[i])}/{args.trials} {filename}')
    if len(missing_losses) > 0:
        print('missing', missing_losses)

    if not args.execute:
        return
    for i in range(len(xs)):
        missing = max(0, args.trials - len(ys[i]))
        loss = f'{i*0.1:.1f}'
        if missing == 0:
            continue
        cmd = ['sudo', '-E', 'python3', 'mininet/net.py', '-n', args.n,
               '--loss2', loss, '-t', str(missing), '--benchmark', key]
        cmd += args.args
        print(' '.join(cmd))
        p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT)
        with open(filename, 'ab') as f:
            for line in p.stdout:
                f.write(line)
                sys.stdout.buffer.write(line)
                sys.stdout.buffer.flush()

def plot_graph(data, pdf):
    max_x = 0
    for (i, key) in enumerate(KEYS):
        (xs, ys) = data[key]
        plt.plot(xs, ys, marker=MARKERS[i], label=key)
        max_x = max(max_x, max(xs))
    plt.xlabel('Loss (%)')
    plt.ylabel('Tput (MB/s)')
    plt.xlim(0, max_x)
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.4), ncol=2)
    plt.title(pdf)
    print(pdf)
    save_pdf(pdf)
    plt.clf()

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true',
        help='whether to execute benchmarks to collect missing data points')
    parser.add_argument('-n', default='20M',
        help='data size (default: 20M)')
    parser.add_argument('-t', '--trials', default=5, type=int,
        help='number of trials per data point (default: 5)')
    parser.add_argument('--bw', default=100, type=int,
        help='bandwidth of near subpath link in Mbps (default: 100)')
    parser.add_argument('--max-x', default=20, type=int,
        help='maximum loss perecentage in tenths of a percentage (default: 20)')
    parser.add_argument('--args', action='extend', nargs='+', default=[],
        help='additional arguments to append to the mininet/net.py command if executing.')
    args = parser.parse_args()

    # Create the directory that holds the results.
    path = f'{WORKDIR}/results/loss_tput/bw{args.bw}/{args.n}'
    os.system(f'mkdir -p {path}')

    # Parse results data, and collect missing data points if specified.
    data = {}
    for key in KEYS:
        filename = f'{path}/{key}.txt'
        print(filename)
        os.system(f'touch {filename}')
        maybe_collect_missing_data(filename, key, args)
        (xs, ys) = parse_data(filename, args.trials, args.max_x)
        new_xs = []
        new_ys = []
        for i in range(len(ys)):
            if len(ys[i]) == 0:
                continue
            new_xs.append(0.1*xs[i])
            new_ys.append(collect_ys(ys[i], args.n))
        data[key] = (new_xs, new_ys)

    # Plot data.
    pdf = f'loss_bw{args.bw}_{args.n}.pdf'
    plot_graph(data, pdf=pdf)
