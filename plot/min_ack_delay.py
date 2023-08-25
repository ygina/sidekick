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

TARGET_XS = [x for x in range(0, 100, 10)] + [x for x in range(100, 1100, 100)]
WORKDIR = os.environ['HOME'] + '/sidecar'

def collect_ys_mean(ys):
    y = statistics.mean(ys)
    yerr = 0 if len(ys) == 1 else statistics.stdev(ys)
    return (y, yerr)

def collect_ys_median(ys):
    y = statistics.median(ys)
    mid = int(len(ys) / 2)
    if len(ys) % 2 == 1:
        p25 = statistics.median(ys[:mid+1])
    else:
        p25 = statistics.median(ys[:mid])
    p75 = statistics.median(ys[mid:])
    yerr = (y-p25, p75-y)
    return (y, yerr)

def parse_data(filename, key, trials, max_x, n, data_key='time_total'):
    """
    Returns (xs, [[data_tput]], [[data_pkts]]), where each the xs are the
    min ack delay, and the values are the time_total converted to goodput
    and the h1-eth0 tx_packets, respectively.
    The maximum min-ack-delay is <= max_x.
    The length of the arrays are <= trials.
    """
    min_ack_delay = None
    exitcode = None
    key_index = None
    exitcode_index = None
    data_tput = defaultdict(lambda: [])
    data_pkts = defaultdict(lambda: [])

    with open(filename) as f:
        lines = f.read().split('\n')
    for line in lines:
        line = line.strip()

        # Get the current min_ack_delay
        m = re.search(r'sudo -E python3 mininet/main\.py.*--min-ack-delay (\d+)', line)
        if m is not None:
            min_ack_delay = int(m.group(1))
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

        # Either we're done with this min_ack_delay or read another data point
        if line == '' or '***' in line or '/tmp' in line or 'No' in line or \
            'factor' in line or 'unaccounted' in line:
            continue
        elif '[sidecar] h1-eth0' in line and exitcode == 0:
            line = line.split()
            data_pkts[min_ack_delay].append(int(line[2]))
            min_ack_delay = None
            exitcode = None
        elif min_ack_delay is not None and exitcode is None:
            line = line.split()
            if len(line) < exitcode_index:
                continue
            try:
                exitcode = int(line[exitcode_index])
            except:
                exitcode = None
            if exitcode != 0:
                exitcode = None
                continue
            data_tput[min_ack_delay].append(time_to_tput(float(line[key_index]), n))

    xs = [x for x in filter(lambda x: x <= max_x, TARGET_XS)]
    xs.sort()
    ys_tput = []
    ys_pkts = []
    for x in xs:
        length = min(len(data_tput[x]), len(data_pkts[x]), trials)
        ys_tput.append(data_tput[x][:length])
        ys_pkts.append(data_pkts[x][:length])
    return (xs, ys_tput, ys_pkts)

def maybe_collect_missing_data(filename, key, args):
    (xs, ys_tput, ys_pkts) = parse_data(filename, key, args.trials, args.max_x, args.n)

    missing_keys = []
    for i, min_ack_delay in enumerate(xs):
        num_missing = max(0, args.trials - len(ys_tput[i]))
        if num_missing == args.trials:
            missing_keys.append(min_ack_delay)
        elif num_missing > 0:
            print(f'min_ack_delay={min_ack_delay} {len(ys_tput[i])}/{args.trials} {filename}')
    if len(missing_keys) > 0:
        print('missing', missing_keys)

    if not args.execute:
        return
    for i, min_ack_delay in enumerate(xs):
        num_missing = max(0, args.trials - len(ys_tput[i]))
        for _ in range(num_missing):
            cmd = ['sudo', '-E', 'python3', 'mininet/main.py', '--delay1', '1',
                   '--delay2', '25', '--bw1', '100', '--bw2', '10', '-t', '1',
                   '--loss1', '0', '--loss2', str(args.loss),
                   '-n', args.n, '--print-statistics',
                   '--frequency', args.frequency,
                   '--threshold', str(args.threshold),
                   '--min-ack-delay', str(min_ack_delay)]
            if 'quack' in key:
                cmd += ['--timeout', '60']
            cmd += [key]
            print(' '.join(cmd))
            p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT)
            with open(filename, 'ab') as f:
                f.write(bytes(' '.join(cmd) + '\n', 'utf-8'))
                for line in p.stdout:
                    f.write(line)
                    sys.stdout.buffer.write(line)
                    sys.stdout.buffer.flush()

def plot_graph(data, https, legend, max_x, ylabel, xlabel='min_ack_delay', ylim=None, pdf=None):
    max_x = max_x
    max_y = 0
    plt.figure(figsize=(8, 6))
    for (i, key) in enumerate(https):
        (xs, ys, yerr) = data[key]
        max_y = max(max_y, max(ys))
        if key in LABEL_MAP:
            label = LABEL_MAP[key]
        else:
            label = key
        if yerr is None:
            plt.plot(xs, ys, marker=MARKERS[i], label=label)
        else:
            plt.errorbar(xs, ys, yerr=yerr, marker=MARKERS[i], label=label)
        if len(xs) > 0:
            max_x = max(max_x, max(xs))
    plt.xlabel(xlabel)
    plt.ylabel(ylabel)
    plt.xlim(0)
    if ylim is None:
        plt.ylim(0)
    else:
        plt.ylim(0, ylim)
    if legend:
        plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.3), ncol=2)
    plt.title(pdf)
    if pdf:
        save_pdf(f'{WORKDIR}/plot/graphs/{pdf}')

def collect_data(xs, ys, median):
    """
    ys is an array of arrays. Collect them so it's just an array.
    Return (xs, ys, errs).
    """
    new_xs = []
    new_ys = []
    if median:
        new_yerrs = ([], [])
    else:
        new_yerrs = []
    for i, x in enumerate(xs):
        if len(ys[i]) == 0:
            continue
        new_xs.append(x)
        if median:
            (collected_ys, yerr) = collect_ys_median(ys[i])
            new_ys.append(collected_ys)
            new_yerrs[0].append(yerr[0])
            new_yerrs[1].append(yerr[1])
        else:
            (collected_ys, yerr) = collect_ys_mean(ys[i])
            new_ys.append(collected_ys)
            new_yerrs.append(yerr)
    return (new_xs, new_ys, new_yerrs)

if __name__ == '__main__':
    DEFAULT_PROTOCOLS = ['quack', 'quic']

    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true',
        help='whether to execute benchmarks to collect missing data points')
    parser.add_argument('-n', default='10M',
        help='data size (default: 10M)')
    parser.add_argument('--http', action='extend', nargs='+', default=[],
        help=f'HTTP versions. (default: {DEFAULT_PROTOCOLS})')
    parser.add_argument('-t', '--trials', default=1, type=int,
        help='number of trials per data point (default: 1)')
    parser.add_argument('--loss', default='0', type=str,
        help='Loss percentage on the near subpath (default: 0)')
    parser.add_argument('--max-x', default=1000, type=int,
        help='maximum minimum ack delay to plot (default: 1000)')
    parser.add_argument('--frequency', default='10ms',
        help='quack frequency (default: 10ms)')
    parser.add_argument('--threshold', default=40, type=int,
        help='quack threshold (default: 40)')
    parser.add_argument('--median', action='store_true',
        help='use the median instead of the mean')
    parser.add_argument('--legend', type=bool, default=True,
        help='Whether to plot a legend [0|1]. (default: 1)')
    args = parser.parse_args()

    # Create the directory that holds the results.
    https = DEFAULT_PROTOCOLS if len(args.http) == 0 else args.http
    path = f'{WORKDIR}/results/min_ack_delay/{args.n}/loss{args.loss}'
    os.system(f'mkdir -p {path}')

    # Parse results data, and collect missing data points if specified.
    data_tput = {}
    data_pkts = {}
    data_all = {}
    for key in https:
        filename = f'{path}/{key}.txt'
        print(filename)
        os.system(f'touch {filename}')
        maybe_collect_missing_data(filename, key, args)
        (xs, ys_tput, ys_pkts) = parse_data(filename, key, args.trials, args.max_x, args.n)
        data_tput[key] = collect_data(xs, ys_tput, args.median)
        data_pkts[key] = collect_data(xs, ys_pkts, args.median)
        data_all[key] = (data_tput[key][1], data_pkts[key][1], None)

    # Plot data.
    pdf = f'min_ack_delay_loss{args.loss}_{args.n}'
    plot_graph(data_pkts, https, args.legend, args.max_x, ylabel='h1-eth0 tx_packets', pdf=f'{pdf}_tx_packets.pdf')
    plot_graph(data_tput, https, args.legend, args.max_x, ylabel='Goodput (Mbit/s)', pdf=f'{pdf}_goodput.pdf')
    plot_graph(data_all, https, args.legend, args.max_x, xlabel='Goodput (Mbit/s)', ylabel='h1-eth0 tx_packets', pdf=f'{pdf}.pdf', ylim=1000)
