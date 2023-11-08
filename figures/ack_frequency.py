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

MAX_THRESHOLD = 350
TARGET_XS = {}
TARGET_XS['retx'] = [x for x in range(0, 20, 2)] + \
                    [x for x in range(20, 500, 10)]
TARGET_XS['ackr'] = [x for x in range(0, 20, 2)] + \
                    [x for x in range(20, 500, 10)]
WORKDIR = os.environ['HOME'] + '/sidecar'

def calculate_threshold(frequency, bdp_multiplier):
    return math.ceil(frequency * 0.833 * bdp_multiplier)

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

def parse_data(filename, key, trials, max_x, n, exp):
    """
    Returns (xs, ys), where each the xs are the frequency, and the values
    are the time_total converted to goodput.
    The maximum frequency is <= max_x.
    The length of the arrays are <= trials.
    """
    frequency = None
    key_index = None
    exitcode_index = None
    data = defaultdict(lambda: [])

    with open(filename) as f:
        lines = f.read().split('\n')
    for line in lines:
        line = line.strip()

        # Get the current frequency
        m = None
        if key.protocol == 'quic':
            m = re.search(r'sudo -E python3 mininet/main\.py.*--min-ack-delay (\d+)', line)
        elif key.protocol == 'quack':
            if 'sudo -E python3 mininet/main.py' in line:
                if 'quic' in line:
                    key_index = None
                    frequency = 0
                    continue
                m = re.search(r'--frequency (\d+)', line)
        if m is not None:
            key_index = None
            frequency = int(m.group(1))
            continue

        # Figure out which index to parse the total time and exitcode
        if 'time_total' in line:
            keys = line.split()
            for i in range(len(keys)):
                if keys[i] == 'time_total':
                    key_index = i
                elif keys[i] == 'exitcode':
                    exitcode_index = i

            continue
        if key_index is None:
            continue

        # Either we're done with this frequency or read another data point
        if '[sidecar]' in line and 'tx_packets' in line:
            key_index = None
            continue
        else:
            line = line.split()
            if len(line) < exitcode_index:
                continue
            if int(line[exitcode_index]) != 0:
                continue
            data[frequency].append(time_to_tput(float(line[key_index]), n))

    xs = []
    for x in TARGET_XS[exp]:
        if x > max_x:
            continue
        if key.protocol == 'quack':
            threshold = calculate_threshold(x, key.bdp_multiplier)
            if threshold > MAX_THRESHOLD:
                continue
        xs.append(x)
    xs.sort()
    ys = []
    for x in xs:
        length = min(len(data[x]), trials)
        ys.append(data[x][:length])
    return (xs, ys)

def maybe_collect_missing_data(filename, key, args):
    (xs, ys) = parse_data(filename, key, args.trials, args.max_x, args.n, args.exp)

    missing_freqs = []
    for i, frequency in enumerate(xs):
        num_missing = max(0, args.trials - len(ys[i]))
        if num_missing == args.trials:
            missing_freqs.append(frequency)
        elif num_missing > 0:
            print(f'{frequency}ms {len(ys[i])}/{args.trials} {filename}')
    if len(missing_freqs) > 0:
        print('missing', missing_freqs)

    if not args.execute:
        return
    for i, frequency in enumerate(xs):
        num_missing = max(0, args.trials - len(ys[i]))
        for _ in range(num_missing):
            cmd = ['sudo', '-E', 'python3', 'mininet/main.py',
                   '--delay1', str(args.delay1), '--delay2', str(args.delay2),
                   '--bw1', str(args.bw1), '--bw2', str(args.bw2), '-t', '1',
                   '--loss1', str(args.loss1), '--loss2', str(args.loss2),
                   '-n', args.n]
            if key.protocol == 'quic':
                cmd += ['--min-ack-delay', str(frequency), 'quic']
            elif key.protocol == 'quack' and frequency == 0:
                cmd += ['--min-ack-delay', str(key.delay), 'quic']
            elif key.protocol == 'quack' and frequency != 0:
                threshold = calculate_threshold(frequency, key.bdp_multiplier)
                cmd += ['--min-ack-delay', str(key.delay),
                        '--frequency', f'{frequency}ms',
                        '--threshold', str(threshold), 'quack']
            print(' '.join(cmd))
            p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT)
            with open(filename, 'ab') as f:
                f.write(bytes(' '.join(cmd) + '\n', 'utf-8'))
                for line in p.stdout:
                    f.write(line)
                    sys.stdout.buffer.write(line)
                    sys.stdout.buffer.flush()

def plot_graph(data, keys, legend, max_x, pdf=None):
    max_x = max_x
    plt.figure(figsize=(9, 6))
    for (i, key) in enumerate(keys):
        (xs, ys, yerr) = data[key.name()]
        if key.name() in LABEL_MAP:
            label = LABEL_MAP[key.name()]
        else:
            label = key.name()
        try:
            plt.errorbar(xs, ys, yerr=yerr, marker=MARKERS[i], label=label)
        except:
            import pdb; pdb.set_trace()
        if len(xs) > 0:
            max_x = max(max_x, max(xs))
    plt.xlabel('quack frequency (ms)')
    plt.ylabel('goodput (Mbit/s)')
    plt.xlim(0, max_x)
    plt.ylim(0)
    if legend:
        plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.4), ncol=2)
    plt.title(pdf)
    if pdf:
        save_pdf(f'{WORKDIR}/plot/graphs/{pdf}')
    plt.clf()

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


class Key:
    def __init__(self, protocol, bdp=None, delay=None):
        self.protocol = protocol
        self.bdp_multiplier = bdp
        self.delay = delay

    def name(self):
        if self.bdp_multiplier is None or self.delay is None:
            return self.protocol
        else:
            return f'{self.protocol}_{self.bdp_multiplier}bdp_delay{self.delay}'


def get_keys(protocols, bdp_multipliers, min_ack_delays):
    keys = []
    if 'quack' in protocols:
        for bdp in bdp_multipliers:
            for delay in min_ack_delays:
                keys.append(Key('quack', bdp=bdp, delay=delay))
    if 'quic' in protocols:
        keys.append(Key('quic'))
    return keys


if __name__ == '__main__':
    DEFAULT_PROTOCOLS = ['quack']
    DEFAULT_BDP_MULTIPLIERS = [1, 4, 0.25]
    DEFAULT_MIN_ACK_DELAYS = [200, 400, 800]

    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(required=True)
    parser.add_argument('--execute', action='store_true',
        help='whether to execute benchmarks to collect missing data points')
    parser.add_argument('-n', default='10M',
        help='data size (default: 10M)')
    parser.add_argument('-t', '--trials', default=1, type=int,
        help='number of trials per data point (default: 1)')
    parser.add_argument('--max-x', type=int,
        help='maximum minimum ack delay to plot')
    parser.add_argument('--median', action='store_true',
        help='use the median instead of the mean')
    parser.add_argument('--legend', type=bool, default=True,
        help='Whether to plot a legend [0|1]. (default: 1)')

    retx = subparsers.add_parser('retx')
    retx.set_defaults(exp='retx', delay1='75', delay2='1', bw1='10', bw2='100',
                      loss1='0', loss2='1',
                      protocols=['quack'],
                      bdp_multipliers=[1, 2, 4, 0.25],
                      min_ack_delays=[0])

    ackr = subparsers.add_parser('ackr')
    ackr.set_defaults(exp='ackr', delay1='1', delay2='40', bw1='100', bw2='10',
                      loss1='0', loss2='0',
                      protocols=['quack', 'quic'],
                      bdp_multipliers=[1, 2, 4, 0.25],
                      min_ack_delays=[800])

    args = parser.parse_args()

    if args.max_x is None:
        if args.exp == 'retx':
            args.max_x = 250
        elif args.exp == 'ackr':
            args.max_x = 200

    # Create the directory that holds the results.
    path = f'{WORKDIR}/results/ack_frequency/{args.exp}/{args.n}'
    os.system(f'mkdir -p {path}')

    # Parse results data, and collect missing data points if specified.
    data = {}
    keys = get_keys(args.protocols, args.bdp_multipliers, args.min_ack_delays)
    for key in keys:
        filename = f'{path}/{key.name()}.txt'
        print(filename)
        os.system(f'touch {filename}')
        maybe_collect_missing_data(filename, key, args)
        (xs, ys) = parse_data(filename, key, args.trials, args.max_x, args.n, args.exp)
        data[key.name()] = collect_data(xs, ys, args.median)

    # Plot data.
    pdf = f'ack_frequency_{args.exp}_{args.n}.pdf'
    plot_graph(data, keys, args.legend, args.max_x, pdf=pdf)
