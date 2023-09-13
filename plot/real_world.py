import argparse
import subprocess
import os
import sys
import re
import os.path
import statistics
import numpy as np
from os import path
from collections import defaultdict
from common import *


DEFAULT_PROTOCOLS_RETX = ['quic', 'quack']
DEFAULT_PROTOCOLS_WEBRTC = ['base', 'quack']
DEFAULT_DATA_SIZES = [1000, 10000, 50000]

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
            data[bm][n] = DataPoint([n / 1000. * 8 / x for x in data[bm][n]])

    plt.clf()
    plt.figure(figsize=(6, 4.5))
    original_xs = np.arange(len(data_sizes))
    width = 0.4

    fig, ax = plt.subplots()
    for (i, bm) in enumerate(https):
        xs = original_xs - 3.*width/2 + width * i
        ys = [data[bm][n].p50 for n in data_sizes]
        if len(ys) == 1:
            yerrs = [0 for _ in data_sizes]
        else:
            yerr_lower = [data[bm][n].p50 - data[bm][n].p25 for n in data_sizes]
            yerr_upper = [data[bm][n].p75 - data[bm][n].p50 for n in data_sizes]
            yerrs = (yerr_lower, yerr_upper)
        label = ['QUIC E2E', 'Sidekick'][i]
        bars = ax.bar(xs, ys, width, label=label, yerr=yerrs, capsize=5,
                      color=COLOR_MAP[bm], fill=True, hatch=HATCHES[i])

    ax.set_xlabel('Upload Data Size (MByte)', fontsize=FONTSIZE)
    ax.set_xticks(original_xs, [f'{int(x / 1000)}MB' for x in data_sizes],
        fontsize=FONTSIZE)
    ax.tick_params(axis='both', which='major', labelsize=FONTSIZE)
    ax.tick_params(axis='both', which='minor', labelsize=FONTSIZE)
    ax.legend(loc='upper center', bbox_to_anchor=(0.5, 1.15), ncol=2,
        fontsize=FONTSIZE)
    ax.set_ylabel('Goodput (Mbit/s)', fontsize=FONTSIZE)
    ax.set_ylim(0)
    ax.grid(axis='y')
    if pdf is not None:
        save_pdf(f'{args.workdir}/plot/graphs/{pdf}')

def parse_data_cdf(args, filename):
    with open(f'{args.workdir}/{filename}') as f:
        lines = f.read().split('\n')

    # Every other trial is base or quack, starting with base.
    # Collect all raw values.
    raw_data = defaultdict(lambda: [])
    for line in lines:
        match = re.match(r'Raw values = \[(.+)\]', line)
        if match is None:
            continue
        values = [int(x) for x in list(match.group(1).split(', '))]
        if len(raw_data['base']) > len(raw_data['quack']):
            raw_data['quack'] += values
        else:
            raw_data['base'] += values
    raw_data['base'].sort()
    raw_data['quack'].sort()

    # Represents args.min_x/10% to 100.0% by 0.1% increments
    key_data = defaultdict(lambda: [])
    for percentile in range(args.min_x, 1001, 1):
        for key in ['base', 'quack']:
            index = int(percentile / 1000.0 * len(raw_data[key]))
            index = min(index, len(raw_data[key]) - 1)
            key_data[key].append(raw_data[key][index])
    return key_data

def plot_webrtc_graph(args, data,
                      keys=['base', 'quack'],
                      labels=['Simple E2E', 'Sidekick'],
                      pdf='real_world_webrtc.pdf'):
    plt.clf()
    plt.figure(figsize=(6, 4.8))
    xs = [x / 10.0 for x in range(args.min_x, 1001)]
    for (i, key) in enumerate(keys):
        ys = [y / 1000000.0 for y in data[key]]
        plt.plot(ys, xs, label=labels[i],
                 linewidth=LINEWIDTH, linestyle=LINESTYLES[i],
                 color=MAIN_RESULT_COLORS[i], markersize=MARKERSIZE)
    plt.ylabel('Percentile', fontsize=FONTSIZE)
    plt.xlabel('De-Jitter Latency (ms)', fontsize=FONTSIZE)
    plt.xticks(fontsize=FONTSIZE)
    min_x = int(args.min_x / 10)
    ticks = [x for x in range(min_x, 102, 2)]
    plt.yticks(ticks=ticks,
               labels=[f'{tick}%' for tick in ticks],
               fontsize=FONTSIZE)
    plt.grid()
    plt.ylim(min(xs), max(xs))
    plt.xlim(0)
    plt.ylim(min_x, 100.5)
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.15), ncol=2, fontsize=FONTSIZE)
    if pdf:
        save_pdf(f'{args.workdir}/plot/graphs/{pdf}')

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--legend', type=bool, default=True,
                        help='Whether to plot a legend [0|1]. (default: 1)')
    parser.add_argument('--min-x', type=int, default=800, help='(default: 800)')
    parser.add_argument('--workdir',
                        default=os.environ['HOME'] + '/sidecar',
                        help='Working directory (default: $HOME/sidecar)')
    parser.add_argument('--cdf-filename', default='raw_data_server_2:11PM')
    args = parser.parse_args()

    plot_retx_graph(args)
    cdf_data = parse_data_cdf(args, filename=f'scripts/webrtc/{args.cdf_filename}')
    plot_webrtc_graph(args, cdf_data)
