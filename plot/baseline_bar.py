import argparse
import subprocess
import os
import sys
import os.path
import statistics
import numpy as np
from os import path
from common import *


def get_filename(loss, bm):
    return f'../results/loss{loss}p/{bm}.txt'

def parse_data(args, filename, bm, data_sizes, data_key='time_total'):
    data = {}
    for n in data_sizes:
        data[n] = []
    if not path.exists(filename):
        return data
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

        results_file = f'{args.workdir}/results/loss{loss}p/{bm}.txt'
        subprocess.Popen(['mkdir', '-p', f'results/loss{loss}p'],
                         cwd=args.workdir).wait()
        cmd = ['sudo', '-E', 'python3', 'mininet/main.py', '--loss2', str(loss),
            '-t', str(missing), '-n', f'{n}k', bm]
        print(cmd)
        p = subprocess.Popen(cmd, cwd=args.workdir, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT)
        with open(results_file, 'ab') as f:
            for line in p.stdout:
                sys.stdout.buffer.write(line)
                sys.stdout.buffer.flush()
                f.write(line)
        p.wait()

def plot_graph(args, loss, data_sizes, https, legend, pdf=None):
    data = {}
    for bm in https:
        filename = get_filename(loss, bm)
        print(filename)
        maybe_collect_missing_data(args, filename, bm, data_sizes)
        data[bm] = parse_data(args, filename, bm, data_sizes)
        for n in data[bm]:
            data[bm][n] = DataPoint(data[bm][n], normalize=8*n/1000.)

    plt.clf()
    original_xs = np.arange(len(data_sizes))
    width = 0.2

    fig, ax = plt.subplots()
    for (i, bm) in enumerate(https):
        xs = original_xs - 3.*width/2 + width * i
        ys = [data[bm][n].p50 for n in data_sizes]
        if args.trials == 1:
            yerrs = [0 for _ in data_sizes]
        else:
            yerrs = [[data[bm][n].p50 - data[bm][n].p25 for n in data_sizes],
                     [data[bm][n].p75 - data[bm][n].p50 for n in data_sizes]]
        bars = ax.bar(xs, ys, width, label=LABEL_MAP[bm], yerr=yerrs,
                      color=COLOR_MAP[bm], fill=True, hatch=HATCHES[i],
                      capsize=5)
        # if statistics.mean(ys) < 2:
        #     ax.bar_label(bars, padding=6, fmt='%1.3f', rotation=90,
        #                  fontsize=FONTSIZE, color='black')
        # else:
        #     ax.bar_label(bars, label_type='center', fmt='%1.3f', rotation=90,
        #                  fontsize=FONTSIZE, color='black')

    ax.set_xlabel('Upload Data Size (MByte)', fontsize=FONTSIZE)
    ax.set_xticks(original_xs, [f'{int(x / 1000)}MB' for x in data_sizes],
        fontsize=FONTSIZE)
    ax.tick_params(axis='both', which='major', labelsize=FONTSIZE)
    ax.tick_params(axis='both', which='minor', labelsize=FONTSIZE)
    if legend:
        ax.legend(loc='upper center', bbox_to_anchor=(0.5, 1.25), ncol=2,
            fontsize=FONTSIZE)
    ax.set_ylabel('Goodput (Mbit/s)', fontsize=FONTSIZE)
    ax.set_ylim(0)
    ax.set_yticks(ticks=[0, 2, 4, 6, 8, 10], fontsize=FONTSIZE)
    ax.grid(axis='y')
    if pdf is not None:
        save_pdf(f'{args.workdir}/plot/graphs/{pdf}')

def plot_legend(args, https, pdf='baseline_bar_legend.pdf'):
    pdf = f'{args.workdir}/plot/graphs/{pdf}'
    plt.clf()
    plot_graph(args, loss=0, data_sizes=[1000], https=https, legend=False)
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.3), ncol=4, frameon=True)
    bbox = Bbox.from_bounds(-3.9, 4.45, 14.4, 0.8)
    save_pdf(pdf, bbox_inches=bbox)

if __name__ == '__main__':
    DEFAULT_LOSSES = [0, 1]
    DEFAULT_DATA_SIZES = [1000, 10000, 50000]
    DEFAULT_PROTOCOLS = ['quic', 'quack', 'tcp', 'pep']

    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true',
                        help='Execute benchmarks for missing data points')
    parser.add_argument('--legend', type=int, default=1,
                        help='Whether to plot a legend [0|1]. (default: 1)')
    parser.add_argument('-t', '--trials', default=10, type=int,
                        help='Number of trials to plot (default: 10)')
    parser.add_argument('--loss', action='extend', nargs='+', default=[],
                        type=int,
                        help=f'Loss percentages to plot. '
                             f'(default: {DEFAULT_LOSSES})')
    parser.add_argument('-n', action='extend', nargs='+', default=[],
                        type=int,
                        help=f'Data sizes to plot, in kBytes. '
                             f'(default: {DEFAULT_DATA_SIZES})')
    parser.add_argument('--http', action='extend', nargs='+', default=[],
                        help=f'HTTP versions. (default: {DEFAULT_PROTOCOLS})')
    parser.add_argument('--workdir',
                        default=os.environ['HOME'] + '/sidecar',
                        help='Working directory (default: $HOME/sidecar)')
    args = parser.parse_args()

    losses = DEFAULT_LOSSES if len(args.loss) == 0 else args.loss
    data_sizes = DEFAULT_DATA_SIZES if len(args.n) == 0 else args.n
    https = DEFAULT_PROTOCOLS if len(args.http) == 0 else args.http
    for loss in losses:
        pdf = f'baseline_loss{loss}p.pdf'
        plot_graph(args, loss, data_sizes, https, args.legend, pdf=pdf)
    plot_legend(args, https)
