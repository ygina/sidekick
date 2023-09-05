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


DEFAULT_PROTOCOLS_RETX = ['quic', 'quack']
DEFAULT_PROTOCOLS_WEBRTC = ['base', 'quack']
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
                    https=DEFAULT_PROTOCOLS_RETX,
                    data_sizes=DEFAULT_DATA_SIZES,
                    pdf='real_world_retx.pdf'):
    data = defaultdict(lambda: {})
    # Add the total times, in seconds, to these arrays when collected.
    # The first 10, 10, and 6 data points for each data size were collected
    # on Thursday, August 31, 2023 around 9pm. The remaining data points
    # were collected on Friday, September 1, 2023 starting from around 1pm
    # until 6pm.
    data['quic'][1000] = [6.712636,2.883463,4.179071,5.220343,2.576400,
                          2.234775,4.009079,2.497224,4.440873,4.746018,
                          13.389671,9.764715,9.511431,6.502541,7.067453,
                          9.837232,9.842330,10.709180,4.697543,5.747926]
    data['quic'][10000] = [40.618722,34.904855,37.588759,35.555504,37.585444,
                           38.849286,24.238750,25.890907,28.548892,31.370909,
                           111.061333,87.487611,76.856121,98.890475,142.246060,
                           107.826417,83.303895,77.942539,79.681728,85.138467]
    data['quic'][50000] = [180.519908,147.414645,171.697321,172.523362,181.503244,204.094687,
                           352.225561,359.077364,308.382826,278.952154,295.755555,
                           250.086729,306.970419,306.970419,228.248670,243.750940,
                           270.907587,270.320967,210.285435,242.174202,185.899284,
                           283.109206,182.633970,140.081237,169.195374,]
    data['quack'][1000] = [3.693857,2.675508,4.441190,2.614579,2.366354,
                           2.673024,2.809964,3.433754,2.447262,3.962095,
                           12.499980,6.455725,4.976494,6.257007,9.636542,
                           11.076679,6.106071,10.519761,7.477837,6.874833]
    data['quack'][10000] = [26.679292,24.413014,22.203797,31.751578,25.160763,
                            18.340019,20.842764,21.037658,22.129411,24.691013,
                            100.224426,53.027380,54.012760,115.545369,71.814570,
                            60.217087,80.826787,69.307548,72.103069,74.187037]
    data['quack'][50000] = [109.367024,112.190154,140.979310,147.493014,133.723531,159.736379,
                            249.012636,266.427067,228.397318,238.007580,165.197820,
                            151.744592,181.738691,208.503198,181.961244,175.791914,
                            200.002917,150.696409,200.033671,138.658734,137.012079,
                            172.855983,110.569008,109.664819,103.688615,]

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
    ax.set_ylim(0)
    if pdf is not None:
        save_pdf(f'{args.workdir}/plot/graphs/{pdf}')

def plot_webrtc_graph_box_and_whiskers(data,
                    percentile=99,
                    https=DEFAULT_PROTOCOLS_WEBRTC,
                    pdf='real_world_webrtc.pdf'):
    data = defaultdict(lambda: {})
    # p95 latencies, in ms (just to track)
    data[95]['base'] = []
    data[95]['quack'] = []
    # p99 latencies, in ms
    data[99]['base'] = [0.000,0.000,250.284604,74.535461,0.000,0.000,210.0883,29.448243,95.361871,0.191958,0.000,0.0,0.0,0.0,0.0,0.0,0.0,64.928623,0.0,0.036869]
    data[99]['quack'] = [0.005957,19.922228,0.000,0.000,0.000,104.950924,0.000,10.375127,40.813684,0.023077,0.000,0.0,0.0,0.0,0.0,9.951971,0.0,0.017112,0.0,0.0]

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

def plot_webrtc_graph_cdf(data,
                          min_x=95.0,
                          keys=DEFAULT_PROTOCOLS_WEBRTC,
                          pdf='real_world_webrtc.pdf'):
    data = {}
    # Paste the array from "Latencies (ns) = <array>" representing the
    # 90th to 100th percentiles. Timeout is 10 minutes.
    data['base'] = []
    data['quack'] = []

    xs = [x / 10.0 for x in range(900, 1001)]
    plt.figure(figsize=(9, 6))
    for (i, key) in enumerate(keys):
        ys = [y / 1000000.0 for y in data[key]]
        plt.plot(xs, ys, label=key)
    plt.xlabel('Percentile')
    plt.ylabel('Latency (ms)')
    if min_x is None:
        plt.xlim(min(xs), max(xs))
    else:
        plt.xlim(min_x / 10.0, max(xs))
    plt.ylim(0)
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.4), ncol=2)
    plt.title(pdf)
    if pdf:
        save_pdf(f'{WORKDIR}/plot/graphs/{pdf}')
    plt.clf()

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
