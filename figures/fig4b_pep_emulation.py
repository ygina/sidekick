import argparse
import subprocess
import os
import sys
import re
import os.path
from os import path
from common import *

TARGET_XS = [x for x in range(200, 1100, 200)] + \
            [x for x in range(2000, 20000, 2000)] + \
            [x for x in range(20000, 50000, 5000)] + \
            [50000]
#             [x for x in range(40000, 100000, 10000)]
# TARGET_XS = [x for x in range(200, 1000, 200)] + \
#             [x for x in range(1000, 10000, 1000)] + \
#             [x for x in range(10000, 20000, 2000)] + \
#             [x for x in range(20000, 100000, 5000)] + \
#             [100000]

def create_cmd(loss, http_version, trials, data_size, bw2):
    cmd = ['sudo', '-E', 'python3', 'mininet/main.py',
        '--loss2', str(loss), '--delay1', '25',
        '-t', str(trials), '--bw2', str(bw2), '-n', f'{data_size}k',
        '--stderr', 'error.log']
    match = re.match(r'quack_(.+(ms|p))_(\d+)', http_version)
    if match is not None:
        cmd += ['--frequency', match.group(1)]
        cmd += ['--threshold', match.group(3)]
        cmd += ['quack']
    else:
        cmd += [http_version]
    return cmd

def parse_data(args, loss, http_version, data_key='time_total'):
    """
    Parses the median keyed time and the data size.
    ([data_size], [time_total])
    """
    filename = get_filename(args.logdir, loss, http_version)
    with open(filename) as f:
        lines = f.read().split('\n')
    xy_map = {}
    data_size = None
    key_index = None
    exitcode_index = None
    data = None

    for line in lines:
        line = line.strip()
        if 'Data Size' in line:
            data_size = int(line[11:-1])
            data = []
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
            'factor' in line or 'unaccounted' in line or 'sudo' in line:
            # Done reading data for this data_size
            if len(data) > 0:
                if data_size not in xy_map:
                    xy_map[data_size] = []
                xy_map[data_size].extend(data)
                if len(xy_map[data_size]) > args.trials:
                    # truncated = len(xy_map[data_size]) - args.trials
                    # print(f'{data_size}k truncating {truncated} points')
                    xy_map[data_size] = xy_map[data_size][:args.trials]
            data_size = None
            key_index = None
            data = None
        else:
            # Read another data point for this data_size
            line = line.split()
            if exitcode_index is not None and int(line[exitcode_index]) != 0:
                continue
            data.append(float(line[key_index]))
    print(filename)

    xs = []
    ys = []
    for data_size in xy_map:
        if data_size in TARGET_XS and data_size <= args.max_x:
            xs.append(data_size)
    xs.sort()
    if len(xs) != len(TARGET_XS):
        missing_xs = []
        for x in TARGET_XS:
            if x in xs or x > args.max_x:
                continue
            missing_xs.append(x)
        if args.execute:
            for x in missing_xs:
                cmd = create_cmd(loss, http_version, args.trials, x, args.bw2)
                execute_experiment(cmd, filename, cwd=args.workdir)
        elif len(missing_xs) > 0:
            print(f'missing {len(missing_xs)} xs: {missing_xs}')
    try:
        for i in range(len(xs)):
            x = xs[i]
            y = xy_map[x]
            if len(y) < args.trials:
                missing = args.trials - len(y)
                if args.execute:
                    cmd = create_cmd(loss, http_version, missing, x, args.bw2)
                    execute_experiment(cmd, filename, cwd=args.workdir)
                else:
                    print(f'{x}k missing {missing}/{args.trials}')
            xs[i] /= 1000.  # convert kilobyte to megabyte
            # convert seconds to megabit / s
            y = DataPoint(y, normalize=xs[i]*8)
            # if 'quic' in filename or 'quack' in filename:
            #     if y.stdev is not None and y.stdev > 0.1:
            #         print(f'ABNORMAL x={x} stdev={y.stdev} f={filename}')
            ys.append(y)
    except Exception as e:
        import pdb; pdb.set_trace()
        raise e
    return (xs, ys)

def get_filename(logdir, loss, http):
    """
    Args:
    - loss: <number>
    - http: tcp, quic, pep, quack
    """
    results = f'{logdir}/loss{loss}p'
    filename = f'{results}/{http}.txt'
    os.system(f'mkdir -p {results}')
    os.system(f'touch {filename}')
    return filename

def plot_graph(args, loss, pdf, http_versions,
               data_key='time_total',
               use_median=True,
               marquee_labels=False):
    data = {}
    for http_version in http_versions:
        filename = get_filename(args.logdir, loss, http_version)
        data[http_version] = parse_data(args, loss, http_version,
                                        data_key=data_key)
    plt.clf()
    plt.figure(figsize=(6, 4))
    for (i, label) in enumerate(http_versions):
        if label not in data:
            continue
        (xs, ys_raw) = data[label]
        if use_median:
            ys = [y.p50 for y in ys_raw]
            yerr_lower = [y.p50 - y.p25 for y in ys_raw]
            yerr_upper = [y.p75 - y.p50 for y in ys_raw]
            yerr = (yerr_lower, yerr_upper)
            if marquee_labels:
                label = 'QUIC E2E' if i == 0 else MAIN_RESULT_LABELS[i]
        else:
            ys = [y.avg for y in ys_raw]
            yerr = [y.stdev if y.stdev is not None else 0 for y in ys_raw]
        plt.errorbar(xs, ys, yerr=(yerr_lower, yerr_upper), capsize=5,
            label=label, marker=MARKERS[i], linewidth=LINEWIDTH,
            linestyle=LINESTYLES[i], zorder=MAIN_RESULT_ZORDERS[i],
            color=MAIN_RESULT_COLORS[i], elinewidth=2, markersize=MARKERSIZE)
        # print(label)
        # print(xs)
        # print(ys)
    plt.xlabel('Upload Data Size (MByte)', fontsize=FONTSIZE)
    plt.ylabel('Goodput (Mbit/s)', fontsize=FONTSIZE)
    plt.xticks(fontsize=FONTSIZE)
    plt.yticks(ticks=[0, 2, 4, 6, 8], fontsize=FONTSIZE)
    plt.grid()
    plt.xlim(0, 50)
    plt.ylim(0)
    if args.legend:
        plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.3), ncol=2, fontsize=FONTSIZE)
    if pdf is not None:
        save_pdf(f'{args.outdir}/{pdf}')

if __name__ == '__main__':
    DEFAULT_LOSSES = [0, 1]
    DEFAULT_PROTOCOLS = ['quack', 'pep', 'quic', 'tcp']

    parser.add_argument('-t', '--trials', default=20, type=int,
                        help='Number of trials to plot (default: 20)')
    parser.add_argument('--max-x', default='50000', type=int,
                        help='Maximum x to plot, in kB (default: 50000)')
    parser.add_argument('--loss', action='extend', nargs='+', default=[],
                        type=int,
                        help=f'Loss percentages to plot. '
                             f'(default: {DEFAULT_LOSSES})')
    parser.add_argument('--http', action='extend', nargs='+', default=[],
                        help=f'HTTP versions. (default: {DEFAULT_PROTOCOLS})')
    parser.add_argument('--bw2', type=int, default=100,
                        help='Bandwidth of link 2 (default: 100).')
    parser.add_argument('--mean', action='store_true',
                        help='Plot mean graphs')
    parser.add_argument('--median', action='store_true',
                        help='Plot median graphs')
    parser.add_argument('--marquee', action='store_true',
                        help='Plot the marquee graph Figure 4a.')
    args = parser.parse_args()

    if args.marquee:
        https = ['quic', 'quack_30ms_10', 'quack_60ms_20', 'quack_120ms_40']
        plot_graph(args, loss=1, pdf=f'fig4b_pep_emulation.pdf',
            http_versions=https, use_median=True, marquee_labels=True)

    losses = DEFAULT_LOSSES if len(args.loss) == 0 else args.loss
    https = DEFAULT_PROTOCOLS if len(args.http) == 0 else args.http

    for loss in losses:
        if args.median:
            plot_graph(args, loss=loss, pdf=f'median_loss{loss}p.pdf',
                http_versions=https, use_median=True)
        if args.mean:
            plot_graph(args, loss=loss, pdf=f'mean_loss{loss}p.pdf',
                http_versions=https, use_median=False)
