import argparse
import subprocess
import os
import sys
import os.path
import statistics
import numpy as np
from os import path
from collections import defaultdict
from common import *


DEFAULT_PROTOCOLS = ['quic', 'quack']
DEFAULT_DATA_SIZES = [1000, 10000, 50000]

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

def plot_retx_graph(args,
                    https=DEFAULT_PROTOCOLS,
                    data_sizes=DEFAULT_DATA_SIZES,
                    pdf='real_world_retx.pdf'):
    data = defaultdict(lambda: {})
    # Add the total times, in seconds, to these arrays when collected.
    data['quic'][1000] = [1, 1, 1]
    data['quic'][10000] = [10, 10, 10]
    data['quic'][50000] = [50, 50, 50]
    data['quack'][1000] = [1, 1, 1]
    data['quack'][10000] = [10, 10, 10]
    data['quack'][50000] = [50, 50, 50]

    for bm in https:
        for n in data_sizes:
            data[bm][n] = collect_parsed_data(data[bm][n], n)

    plt.clf()
    original_xs = np.arange(len(data_sizes))
    width = 0.4

    fig, ax = plt.subplots()
    for (i, bm) in enumerate(https):
        xs = original_xs - 3.*width/2 + width * i
        ys = [data[bm][n][0] for n in data_sizes]
        if len(ys) == 1:
            yerrs = [0 for _ in data_sizes]
        else:
            yerrs = [data[bm][n][1] for n in data_sizes]
        bars = ax.bar(xs, ys, width, label=LABEL_MAP[bm], yerr=yerrs)
        if statistics.mean(ys) < 0.25:
            ax.bar_label(bars, padding=6, fmt='%1.3f', rotation=90,
                         fontsize=FONTSIZE, color='black')
        else:
            ax.bar_label(bars, label_type='center', fmt='%1.3f', rotation=90,
                         fontsize=FONTSIZE, color='white')

    for n in data_sizes:
        for bm in https:
            (y, yerr) = data[bm][n]

    ax.set_xlabel('Data Size', fontsize=FONTSIZE)
    ax.set_xticks(original_xs, [f'{int(x / 1000)}MB' for x in data_sizes],
        fontsize=FONTSIZE)
    ax.tick_params(axis='both', which='major', labelsize=FONTSIZE)
    ax.tick_params(axis='both', which='minor', labelsize=FONTSIZE)
    if args.legend:
        ax.legend(loc='upper center', bbox_to_anchor=(0.5, 1.15), ncol=2,
            fontsize=FONTSIZE)
    ax.set_ylabel('Goodput (MBytes/s)', fontsize=FONTSIZE)
    ax.set_ylim(0, 1.2)
    if pdf is not None:
        save_pdf(f'{args.workdir}/plot/graphs/{pdf}')

def plot_webrtc_graph(data,
                    percentile=99,
                    https=DEFAULT_PROTOCOLS,
                    pdf='real_world_webrtc.pdf'):
    data = defaultdict(lambda: {})
    # p95 latencies, in ms (just to track)
    data[95]['quic'] = []
    data[95]['quack'] = []
    # p99 latencies, in ms
    data[99]['quic'] = [1,2,3,4,5,6,7,8,9,10]
    data[99]['quack'] = [1,2,3,4,5,6,7,8,9,10]

    plt.clf()

    protocols = [[y for y in data[percentile][bm]] for bm in https]
    fig, ax = plt.subplots(figsize=(6,4))
    pos = np.arange(len(protocols)) + 1
    bp = ax.boxplot(protocols, sym='k+', positions=pos, notch=False)

    plt.yticks(fontsize=FONTSIZE)
    ax.set_xticks(pos, https, fontsize=FONTSIZE)
    ax.set_xlabel('Protocol', fontsize=FONTSIZE)
    ax.set_ylabel('p99 Latency (ms)', fontsize=FONTSIZE)
    plt.setp(bp['whiskers'], color='k', linestyle='-')
    plt.setp(bp['fliers'], markersize=3.0)

    plt.title(pdf, fontsize=FONTSIZE)
    save_pdf(f'{args.workdir}/plot/graphs/{pdf}')

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--legend', type=bool, default=True,
                        help='Whether to plot a legend [0|1]. (default: 1)')
    parser.add_argument('--workdir',
                        default=os.environ['HOME'] + '/sidecar',
                        help='Working directory (default: $HOME/sidecar)')
    args = parser.parse_args()

    plot_retx_graph(args)
    plot_webrtc_graph(args)
