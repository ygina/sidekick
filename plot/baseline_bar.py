import argparse
import subprocess
import os
import sys
import os.path
import statistics
import numpy as np
from os import path
from common import *

NUM_TRIALS = None
MAX_X = None
DATA_SIZES = [1000, 10000, 100000]
LOSSES = [0, 2, 5]
HTTP_VERSIONS = ['pep', 'quack-2ms-r', 'quic', 'tcp']

def get_filename(loss, bm):
    return f'../results/loss{loss}p/cubic/{bm}.txt'

def parse_data(args, filename, bm, data_sizes, data_key='time_total'):
    data = {}
    for n in data_sizes:
        data[n] = []
    with open(filename) as f:
        lines = f.read().split('\n')

    n = None
    key_index = None
    exitcode_index = None
    for line in lines:
        line = line.strip()
        if 'Data Size' in line:
            n = int(line[11:-1])
            continue
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
        if line == '' or '***' in line or '/tmp' in line or 'No' in line or \
            'factor' in line or 'unaccounted' in line:
            # Done reading data for this n
            n = None
            key_index = None
            exitcode_index = None
        else:
            # Read another data point for this n
            line = line.split()
            if exitcode_index is not None and int(line[exitcode_index]) != 0:
                continue
            if n in data:
                data[n].append(float(line[key_index]))

    for n in data:
        data[n] = data[n][:min(len(data[n]), args.trials)]
    return data

def maybe_collect_missing_data(args, filename, bm, data_sizes,
                               data_key='time_total'):
    data = parse_data(args, filename, bm, data_sizes, data_key=data_key)
    for n in data:
        y = data[n]
        if len(y) >= args.trials:
            continue
        missing = args.trials - len(y)
        if not args.execute:
            print(f'{n}k\t{len(y)}/{args.trials} points')
            continue

        suffix = f'loss{loss}p/cubic'
        results_file = f'{args.workdir}/results/{suffix}/{bm}.txt'
        if bm == 'pep':
            bm_args = ['tcp', '--pep']
        elif 'quack' in bm:
            bm_args = ['quic', '--sidecar', '2ms', '--quack-reset']
        elif 'quic' in bm:
            bm_args = ['quic']
        elif 'tcp' in bm:
            bm_args = ['tcp']
        else:
            bm_args = [bm]
        subprocess.Popen(['mkdir', '-p', f'results/{suffix}'],
                         cwd=args.workdir).wait()
        cmd = ['sudo', '-E', 'python3', 'mininet/net.py', '--loss2', str(loss),
            '--benchmark'] + bm_args + ['-t', str(missing), '-n', f'{n}k']
        print(cmd)
        p = subprocess.Popen(cmd, cwd=args.workdir, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT)
        with open(results_file, 'ab') as f:
            for line in p.stdout:
                sys.stdout.buffer.write(line)
                sys.stdout.buffer.flush()
                f.write(line)
        p.wait()

def collect_parsed_data(data, n):
    data = [n / 1000. / x for x in data]
    if len(data) == 0:
        return (0, 0)
    mean = statistics.mean(data)
    if len(data) > 1:
        stdev = statistics.stdev(data)
    else:
        stdev = 0
    return (mean, stdev)

def plot_graph(args, loss, data_sizes=DATA_SIZES, pdf=None):
    fontsize = 15
    data = {}
    for bm in HTTP_VERSIONS:
        filename = get_filename(loss, bm)
        print(filename)
        maybe_collect_missing_data(args, filename, bm, data_sizes)
        data[bm] = parse_data(args, filename, bm, data_sizes)
        for n in data[bm]:
            data[bm][n] = collect_parsed_data(data[bm][n], n)

    plt.clf()
    original_xs = np.arange(len(data_sizes))
    width = 0.2

    fig, ax = plt.subplots()
    for (i, bm) in enumerate(HTTP_VERSIONS):
        xs = original_xs - 3.*width/2 + width * i
        ys = [data[bm][n][0] for n in data_sizes]
        if args.trials == 1:
            yerrs = [0 for _ in data_sizes]
        else:
            yerrs = [data[bm][n][1] for n in data_sizes]
        bars = ax.bar(xs, ys, width, label=bm, yerr=yerrs)
        if statistics.mean(ys) < 0.25:
            ax.bar_label(bars, padding=6, fmt='%1.3f', rotation=90,
                         fontsize=fontsize, color='black')
        else:
            ax.bar_label(bars, label_type='center', fmt='%1.3f', rotation=90,
                         fontsize=fontsize, color='white')

    for n in data_sizes:
        for bm in HTTP_VERSIONS:
            (y, yerr) = data[bm][n]

    ax.set_xlabel('Data Size', fontsize=fontsize)
    ax.set_xticks(original_xs, [f'{int(x / 1000)}MB' for x in data_sizes],
        fontsize=fontsize)
    ax.legend(loc='upper center', bbox_to_anchor=(0.5, 1.25), ncol=4,
        fontsize=fontsize)
    ax.set_ylabel('Goodput (Mbyte/s)', fontsize=fontsize)
    ax.set_title(f'{loss}% loss on near subpath', fontsize=fontsize+5)
    ax.set_ylim(0, 1.2)
    if pdf is not None:
        print(pdf)
        save_pdf(pdf)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true',
                        help='Execute benchmarks for missing data points')
    parser.add_argument('-t', '--trials', default=10, type=int,
                        help='Number of trials to plot (default: 10)')
    parser.add_argument('--loss', action='extend', nargs='+', default=[],
                        type=int,
                        help='Loss percentages to plot [0|1|2|5]. Multiple '
                             'arguments can be provided. If no argument is '
                             'provided, plots 0, 2, and 5.')
    parser.add_argument('-n', action='extend', nargs='+', default=[],
                        type=int,
                        help='Data sizes to plot, in kBytes. Multiple '
                             'arguments can be provided. If no argument is '
                             'provided, plots 1000, 10000, 100000.')
    parser.add_argument('--workdir',
                        default=os.environ['HOME'] + '/sidecar',
                        help='Working directory (default: $HOME/sidecar)')
    args = parser.parse_args()

    losses = LOSSES if len(args.loss) == 0 else args.loss
    data_sizes = DATA_SIZES if len(args.n) == 0 else args.n
    for loss in losses:
        pdf = f'baseline_loss{loss}p.pdf'
        plot_graph(args, loss=loss, data_sizes=data_sizes, pdf=pdf)
