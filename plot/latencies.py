import argparse
import subprocess
import os
import re
import sys
import os.path
import statistics
import math
import numpy as np
from os import path
from collections import defaultdict
from common import *

WORKDIR = os.environ['HOME'] + '/sidecar'

def plot_percentile_vs_latency_graph(data, keys, xs=range(101), legend=True, pdf=None):
    plt.figure(figsize=(9, 2))
    for (i, key) in enumerate(keys):
        ys = [y / 1000000.0 for y in data[key]]
        plt.plot(xs, ys, marker=MARKERS[i], label=key)
    plt.xlabel('Percentile')
    plt.ylabel('Latency (ms)')
    plt.xlim(min(xs), max(xs))
    plt.ylim(0)
    if legend:
        plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.8), ncol=2)
    plt.title(pdf)
    if pdf:
        save_pdf(f'{WORKDIR}/plot/graphs/{pdf}')
    plt.clf()

def plot_bar_graph(data, percentiles, keys, pdf, trials=1):
    plt.clf()
    width = 0.2
    original_xs = np.arange(len(percentiles))

    fig, ax = plt.subplots(figsize=(12, 3))
    for (i, key) in enumerate(keys):
        xs = original_xs - 3.*width/2 + width * i
        ys = [data[key][pct][0] for pct in percentiles]
        if trials == 1:
            yerrs = [0 for _ in percentiles]
        else:
            yerrs = [data[key][pct][1] for pct in percentiles]
        bars = ax.bar(xs, ys, width, label=key, yerr=yerrs)
        if statistics.mean(ys) < 80:
            ax.bar_label(bars, padding=6, fmt='%1.3f', rotation=90,
                         fontsize=FONTSIZE, color='black')
        else:
            ax.bar_label(bars, label_type='center', fmt='%1.3f', rotation=90,
                         fontsize=FONTSIZE, color='white')

    ax.set_xticks(original_xs, percentiles,
        fontsize=FONTSIZE)
    ax.tick_params(axis='both', which='major', labelsize=FONTSIZE)
    ax.tick_params(axis='both', which='minor', labelsize=FONTSIZE)
    ax.legend(loc='upper center', bbox_to_anchor=(0.5, 1.3), ncol=4,
        fontsize=FONTSIZE)
    ax.set_ylabel('Latency(ms)', fontsize=FONTSIZE)
    ax.set_ylim(0)
    save_pdf(f'{WORKDIR}/plot/graphs/{pdf}')

def plot_box_and_whiskers_graph(data, keys, pdf):
    plt.clf()

    protocols = [[y / 1000 for y in data[key]] for key in keys]
    fig, ax = plt.subplots(figsize=(1.8*len(keys),4))
    pos = np.arange(len(protocols)) + 1
    bp = ax.boxplot(protocols, sym='k+', positions=pos, notch=False)

    plt.yticks(fontsize=FONTSIZE)
    ax.set_xticks(pos, keys, fontsize=FONTSIZE)
    ax.set_xlabel('Protocol', fontsize=FONTSIZE)
    ax.set_ylabel('p99 Latency (ms)', fontsize=FONTSIZE)
    plt.setp(bp['whiskers'], color='k', linestyle='-')
    plt.setp(bp['fliers'], markersize=3.0)

    plt.title(pdf, fontsize=FONTSIZE)
    save_pdf(f'{WORKDIR}/plot/graphs/{pdf}')

def parse_data(filename, pcts, trials):
    data = defaultdict(lambda: [])
    num_points = 0
    with open(filename) as f:
        lines = f.read().split('\n')
    for line in lines:
        match = re.match(r'Latencies \(ns\) = (.+)', line)
        if match is None:
            continue
        # Represents 95.0% to 100.0% by 0.1% increments
        latencies = [int(x) for x in list(match.group(1)[1:-1].split(', '))]
        for pct in pcts:
            assert pct >= 950
            us = int(latencies[pct - 950] / 1000)
            data[pct].append(us)
        num_points += 1
        if num_points >= trials:
            break
    return (num_points, data)

# Should only be one trial
def parse_data_cdf(filename):
    data = defaultdict(lambda: [])
    with open(filename) as f:
        lines = f.read().split('\n')
    for line in lines:
        match = re.match(r'Latencies \(ns\) = (.+)', line)
        if match is None:
            continue
        # Represents 95.0% to 100.0% by 0.1% increments
        return [int(x) for x in list(match.group(1)[1:-1].split(', '))]

def maybe_collect_missing_data(filename, key, args):
    num_points, _ = parse_data(filename, DEFAULT_PERCENTILES, args.trials)
    if num_points >= args.trials:
        print(filename)
        return
    num_missing = args.trials - num_points
    print(f'{filename} MISSING {num_missing}/{args.trials}')
    if not args.execute:
        return

    for _ in range(num_missing):
        cmd = f'sudo -E python3 mininet/webrtc.py --timeout {args.timeout}'
        cmd = cmd.split(' ')
        match = re.match(r'quack_(.+(ms|p))_(\d+)', key)
        if match is not None:
            cmd += ['--frequency', match.group(1)]
            cmd += ['--threshold', match.group(3)]
            cmd += ['quack']
        else:
            cmd += [key]
        print(' '.join(cmd))

        WORKDIR = os.environ['HOME'] + '/sidecar'
        p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT)
        with open(filename, 'ab') as f:
            for line in p.stdout:
                sys.stdout.buffer.write(line)
                sys.stdout.buffer.flush()
                f.write(line)
        p.wait()

if __name__ == '__main__':
    DEFAULT_PDF = f'latencies_webrtc.pdf'
    DEFAULT_KEYS = ['base', 'quack_2p_8', 'quack_4p_16', 'quack_8p_32']
    DEFAULT_PERCENTILES = [990]

    parser = argparse.ArgumentParser()
    parser.add_argument('--line-graph', action='store_true',
        help='Plot the line graph of percentile vs. latency')
    parser.add_argument('--bar-graph', action='store_true',
        help='Plot the bar graphs of configuration vs. p95 or p99 latency')
    parser.add_argument('--box-and-whiskers', action='store_true',
        help='Plot the box and whiskers plot')
    parser.add_argument('--cdf', action='store_true',
        help='Plot the CDF from p99 to p100')
    parser.add_argument('--execute', action='store_true',
        help='whether to execute benchmarks to collect missing data points')
    parser.add_argument('--timeout', default=60, type=int,
        help='Time to run each trial, in seconds (default: 60)')
    parser.add_argument('-t', '--trials', default=10, type=int,
        help='number of trials per data point (default: 1)')
    parser.add_argument('--http', action='extend', nargs='+', default=[],
        help=f'HTTP versions. (default: {DEFAULT_KEYS})')
    parser.add_argument('--percentile', action='extend', nargs='+', default=[], type=int,
        help=f'Percentiles to print. (default: {DEFAULT_PERCENTILES})')
    args = parser.parse_args()

    if args.line_graph:
        data = {}
        keys = ['base_loss1', 'quack_loss1_freq19ms']
        # keys += ['base_loss10', 'quack_loss10_freq20ms']
        data['base_loss1'] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13019159, 32442532, 52598017, 72855292, 92872845, 113088289, 133182839, 152555689]
        data['base_loss10'] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12071625, 12223073, 13089241, 32151668, 32256333, 32351614, 33259203, 52297331, 52400903, 52514110, 52575809, 71627423, 72427185, 72568345, 72677660, 72713662, 91791780, 91817737, 91942029, 92786273, 92821809, 93021003, 111948634, 111984252, 112069136, 112090391, 112933331, 112985412, 125076431, 132105265, 132207011, 132222096, 132237894, 132268316, 133156865, 152337121, 152353566, 152359315, 152367358, 152371263, 152381893, 152397022, 152416960, 172325228, 172656502, 191904652, 204902971, 212051176, 232043276, 232300646, 252173693, 252384804, 272339933, 272582052, 291740321, 292693624, 311870113, 312833912, 332988162, 372545070, 431935188, 472369756, 532996842, 612708734, 792415139]
        data['quack_loss1_freq15ms'] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15321163]
        data['quack_loss1_freq19ms'] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 19248287]
        data['quack_loss1_freq20ms'] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3991917, 4091838, 4956121, 4975895, 4988555, 5009217, 5011281, 5052052, 5089611, 15154229, 15175043]
        data['quack_loss10_freq20ms'] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4121921, 4315239, 5004513, 5019362, 5026590, 5034487, 5077427, 5199125, 15296679, 54856115]
        plot_percentile_vs_latency_graph(data, xs=[x for x in range(101)], keys=keys, pdf=DEFAULT_PDF)
        exit(0)

    keys = DEFAULT_KEYS if len(args.http) == 0 else args.http
    pcts = DEFAULT_PERCENTILES if len(args.percentile) == 0 else args.percentile
    path = f'{WORKDIR}/results/latencies/timeout{args.timeout}'
    os.system(f'mkdir -p {path}')

    if args.bar_graph:
        data = {
            'base': {
                'p95': [72.680],
                'p99': [152.379],
            },
            'quack_1p_4': {
                'p95': [0],
                'p99': [0],
            },
            'quack_2p_8': {
                'p95': [0],
                'p99': [0],
            },
            'quack_4p_16': {
                'p95': [0],
                'p99': [21.393],
            },
            'quack_8p_32': {
                'p95': [0, 0, 2.262854],
                'p99': [82.157, 62.853, 101.28781],
            },
        }
        percentiles = ['p95', 'p99']
        plot_bar_graph(data, percentiles, keys=keys, pdf=DEFAULT_PDF)
        exit(0)

    if args.cdf:
        key_data = {}
        for key in keys:
            filename = f'{path}/{key}.txt'
            os.system(f'touch {filename}')
            _, data = parse_data_cdf(filename)
            if data is not None:
                key_data[key] = data
        plot_percentile_vs_latency_graph(key_data,
                                         xs=[x / 10.0 for x in range(950, 1001)],
                                         keys=keys,
                                         pdf=DEFAULT_PDF)

    for key in keys:
        filename = f'{path}/{key}.txt'
        os.system(f'touch {filename}')
        maybe_collect_missing_data(filename, key, args)
        _, data = parse_data(filename, pcts, args.trials)
        for pct in pcts:
            if len(data[pct]) == 0:
                continue
            data_point = DataPoint(data[pct])
            print(f'{key} p{pct}: {int(data_point.avg)}±{int(data_point.stdev)} us')

    if args.box_and_whiskers:
        for pct in pcts:
            key_data = {}
            for key in keys:
                filename = f'{path}/{key}.txt'
                _, data = parse_data(filename, pcts, args.trials)
                if len(data[pct]) == 0:
                    continue
                key_data[key] = data[pct]
            plot_box_and_whiskers_graph(key_data, keys, pdf=f'latencies_webrtc_p{pct}.pdf')
