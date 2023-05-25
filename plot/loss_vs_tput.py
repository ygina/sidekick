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

TARGET_XS = {}
# [x for x in range(0, 30, 2)] + \
# TARGET_XS['tcp'] =  [x for x in range(0, 20, 5)] + \
#                     [x for x in range(20, 40, 10)] + \
#                     [x for x in range(40, 100, 20)] + \
#                     [x for x in range(100, 1000, 100)]
# TARGET_XS['quic'] = [x for x in range(0, 20, 5)] + \
#                     [x for x in range(20, 40, 10)] + \
#                     [x for x in range(40, 100, 20)] + \
#                     [x for x in range(100, 500, 20)] + \
#                     [x for x in range(500, 1000, 100)]
# # TARGET_XS['tcp'] = [0]
# # TARGET_XS['quic'] = [0]
# TARGET_XS['quack'] = [x for x in range(0, 1000, 100)]
# TARGET_XS['pep'] = TARGET_XS['quack']
TARGET_XS['pep'] = [x for x in range(0, 1000, 100)]
TARGET_XS['quack'] = [x for x in range(0, 1000, 100)]
# TARGET_XS['quic'] = [0, 5, 10, 15, 20, 30, 40, 50, 100]
# TARGET_XS['quic'] += [200, 400, 800]
# TARGET_XS['tcp'] = TARGET_XS['quic']
TARGET_XS['quic'] = [0, 100, 200]
TARGET_XS['tcp'] = [0, 100, 200]

WORKDIR = os.environ['HOME'] + '/sidecar'

def empty_list():
    return []

def collect_ys_mean(ys, n):
    assert n[-1] == 'M'
    n_megabyte = int(n[:-1]) * 1.0
    ys = [n_megabyte / y for y in ys]
    y = statistics.mean(ys)
    yerr = 0 if len(ys) == 1 else statistics.stdev(ys)
    return (y, yerr)

def collect_ys_median(ys, n):
    assert n[-1] == 'M'
    n_megabyte = int(n[:-1]) * 1.0
    ys = [n_megabyte / y for y in ys]
    ys.sort()
    y = statistics.median(ys)
    mid = int(len(ys) / 2)
    if len(ys) % 2 == 1:
        p25 = statistics.median(ys[:mid+1])
    else:
        p25 = statistics.median(ys[:mid])
    p75 = statistics.median(ys[mid:])
    yerr = (y-p25, p75-y)
    return (y, yerr)

def parse_data(filename, key, trials, max_x, data_key='time_total'):
    loss = None
    key_index = None
    exitcode_index = None
    data = defaultdict(empty_list)

    with open(filename) as f:
        lines = f.read().split('\n')
    for line in lines:
        line = line.strip()

        # Get the current loss percentage in hundredths of a percent
        m = re.search(r'Link2.*loss=(\S+) .*', line)
        if m is not None:
            loss = round(float(m.group(1)) * 100.0)
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
            if len(line) < exitcode_index:
                loss = None
                key_index = None
                exitcode_index = None
                continue
            if exitcode_index is not None and int(line[exitcode_index]) != 0:
                continue
            data[loss].append(float(line[key_index]))

    xs = [x for x in filter(lambda x: x <= max_x, TARGET_XS[key])]
    xs.sort()
    ys = [data[x][:min(len(data[x]), trials)] for x in xs]
    return (xs, ys)

def maybe_collect_missing_data(filename, key, args):
    (xs, ys) = parse_data(filename, key, args.trials, args.max_x)

    missing_losses = []
    for i in range(len(xs)):
        missing = max(0, args.trials - len(ys[i]))
        loss = f'{xs[i]*0.01:.2f}'
        if missing == args.trials:
            missing_losses.append(loss)
        elif missing > 0:
            print(f'{loss}% {len(ys[i])}/{args.trials} {filename}')
    if len(missing_losses) > 0:
        print('missing', missing_losses)

    if not args.execute:
        return
    for i in range(len(xs)):
        missing = max(0, args.trials - len(ys[i]))
        loss = f'{xs[i]*0.01:.2f}'
        if missing == 0:
            continue
        cmd = ['sudo', '-E', 'python3', 'mininet/main.py', '-n', args.n,
               '--loss2', loss, '-t', str(missing), '--bw2', str(args.bw),
               '--stderr', 'loss_tput.error']
        cmd += args.args
        cmd += [key]
        print(' '.join(cmd))
        p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT)
        with open(filename, 'ab') as f:
            for line in p.stdout:
                f.write(line)
                sys.stdout.buffer.write(line)
                sys.stdout.buffer.flush()

def plot_graph(data, https, legend, pdf=None):
    max_x = 0
    plt.figure(figsize=(15, 5))
    for (i, key) in enumerate(https):
        (xs, ys, yerr) = data[key]
        if key in LABEL_MAP:
            label = LABEL_MAP[key]
        else:
            label = key
        plt.errorbar(xs, ys, yerr=yerr, marker=MARKERS[i], label=label)
        max_x = max(max_x, max(xs))
    plt.xlabel('Loss (%)')
    plt.ylabel('Goodput (MBytes/s)')
    plt.xlim(0, max_x)
    plt.ylim(0)
    if legend:
        plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.3), ncol=4)
    plt.title(pdf)
    if pdf:
        save_pdf(pdf)

def plot_legend(data, https, pdf):
    plot_graph(data, https, legend=False)
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.3), ncol=4, frameon=True)
    bbox = Bbox.from_bounds(0.5, 4.55, 14.35, 0.95)
    save_pdf(pdf, bbox_inches=bbox)

if __name__ == '__main__':
    DEFAULT_PROTOCOLS = ['quack', 'pep', 'quic', 'tcp']

    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true',
        help='whether to execute benchmarks to collect missing data points')
    parser.add_argument('-n', default='10M',
        help='data size (default: 10M)')
    parser.add_argument('--http', action='extend', nargs='+', default=[],
        help=f'HTTP versions. (default: {DEFAULT_PROTOCOLS})')
    parser.add_argument('-t', '--trials', default=1, type=int,
        help='number of trials per data point (default: 1)')
    parser.add_argument('--bw', default=100, type=int,
        help='bandwidth of near subpath link in Mbps (default: 100)')
    parser.add_argument('--max-x', default=800, type=int,
        help='maximum loss perecentage in hundredths of a percentage (default: 800)')
    parser.add_argument('--args', action='extend', nargs='+', default=[],
        help='additional arguments to append to the mininet/main.py command if executing.')
    parser.add_argument('--median', action='store_true',
        help='use the median instead of the mean')
    parser.add_argument('--legend', type=bool, default=True,
        help='Whether to plot a legend [0|1]. (default: 1)')
    args = parser.parse_args()

    # Create the directory that holds the results.
    https = DEFAULT_PROTOCOLS if len(args.http) == 0 else args.http
    path = f'{WORKDIR}/results/loss_tput/bw{args.bw}/{args.n}'
    os.system(f'mkdir -p {path}')

    # Parse results data, and collect missing data points if specified.
    data = {}
    for key in https:
        filename = f'{path}/{key}.txt'
        print(filename)
        os.system(f'touch {filename}')
        maybe_collect_missing_data(filename, key, args)
        (xs, ys) = parse_data(filename, key, args.trials, args.max_x)
        new_xs = []
        new_ys = []
        if args.median:
            new_yerrs = ([], [])
        else:
            new_yerrs = []
        for i in range(len(ys)):
            if len(ys[i]) == 0:
                continue
            new_xs.append(0.01*xs[i])
            if args.median:
                (collected_ys, yerr) = collect_ys_median(ys[i], args.n)
                new_ys.append(collected_ys)
                new_yerrs[0].append(yerr[0])
                new_yerrs[1].append(yerr[1])
            else:
                (collected_ys, yerr) = collect_ys_mean(ys[i], args.n)
                new_ys.append(collected_ys)
                new_yerrs.append(yerr)
        data[key] = (new_xs, new_ys, new_yerrs)

    # Plot data.
    pdf = f'loss_bw{args.bw}_{args.n}.pdf'
    plot_graph(data, https, args.legend, pdf=pdf)
    plot_legend(data, https, pdf='legend.pdf')
