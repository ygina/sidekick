import os
import re
from common import *

def plot_percentile_vs_latency_graph_flipped(args, data, keys, min_x=None, xs=range(101), pdf=None):
    plt.figure(figsize=(6, 4))
    zorders = [2, 3, 1, 0]
    for (i, key) in enumerate(keys):
        ys = [y / 1000000.0 for y in data[key]]
        label = 'Simple E2E' if i == 0 else MAIN_RESULT_LABELS[i]
        plt.plot(ys, xs, label=label, linewidth=LINEWIDTH,
                 linestyle=LINESTYLES[i], zorder=MAIN_RESULT_ZORDERS[i],
                 color=MAIN_RESULT_COLORS[i], markersize=MARKERSIZE)
    plt.ylabel('Percentile', fontsize=FONTSIZE)
    plt.xlabel('De-Jitter Latency (ms)', fontsize=FONTSIZE)
    plt.xticks(fontsize=FONTSIZE)
    min_x = min(xs) if min_x is None else int(min_x / 10.0)
    ticks = [tick for tick in range(min_x, 102, 2)]
    plt.yticks(ticks=ticks,
               labels=[f'{tick}%' for tick in ticks],
               fontsize=FONTSIZE)
    plt.grid()
    plt.ylim(min_x, 100.5)
    plt.xlim(0)
    if args.legend:
        plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.3), ncol=2, fontsize=FONTSIZE)
    if pdf:
        save_pdf(f'{args.outdir}/{pdf}')
    plt.clf()

# Parse 5 1-minute trials
def parse_data_cdf(filename, args):
    with open(filename) as f:
        lines = f.read().split('\n')
    num_trials = 0
    raw_data = []
    for line in lines:
        match = re.match(r'Raw values = (.+)', line)
        if match is None:
            continue
        raw_data += [int(x) for x in list(match.group(1)[1:-1].split(', '))]
        num_trials += 1
        if num_trials == args.trials:
            break
    if num_trials < args.trials:
        return (None, num_trials)
    # Now that we have all the raw data points, get the min_x-th
    # to 100th percentiles by 0.1% increments.
    raw_data.sort()
    data = []
    for percentile in range(args.min_x, 1001, 1):
        index = int(percentile / 1000.0 * len(raw_data))
        index = min(index, len(raw_data) - 1)
        data.append(raw_data[index])
    return (data, num_trials)

def maybe_collect_missing_data(filename, key, args):
    data, num_trials = parse_data_cdf(filename, args)
    if data is not None:
        print(filename)
        return data
    if not args.execute:
        print(f'{filename} {num_trials}/{args.trials}')
        return

    cmd = ['sudo', '-E', 'python3', 'mininet/webrtc.py']
    cmd += ['--timeout', str(args.timeout)]
    if args.flip:
        cmd += ['--loss1', str(args.loss), '--loss2', '0']
        cmd += ['--delay1', '1', '--delay2', '25']
        cmd += ['--bw1', '100', '--bw2', '10']
    else:
        cmd += ['--loss2', str(args.loss)]
    match = re.match(r'quack_(.+(ms|p))_(\d+)', key)
    if match is not None:
        cmd += ['--frequency', match.group(1)]
        cmd += ['--threshold', match.group(3)]
        cmd += ['quack']
    else:
        cmd += [key]

    for _ in range(args.trials - num_trials):
        execute_experiment(cmd, filename, cwd=args.workdir)

if __name__ == '__main__':
    DEFAULT_PDF = f'fig4a_low_latency_media.pdf'
    DEFAULT_KEYS = ['base', 'quack_2p_8', 'quack_4p_16', 'quack_8p_32']

    parser.add_argument('--timeout', default=60, type=int,
        help='Time to run a trial, in seconds (default: 60)')
    parser.add_argument('-t', '--trials', default=5, type=int,
        help='number of trials per data point (default: 5)')
    parser.add_argument('--http', action='extend', nargs='+', default=[],
        help=f'HTTP versions. (default: {DEFAULT_KEYS})')
    parser.add_argument('--loss', type=str, default='3.6',
        help='Loss percentage on the near subpath (default: 3.6)')
    parser.add_argument('--min-x', type=int, default=860,
        help='Min x value to plot the CDF at, in tenths of a percentile.')
    parser.add_argument('--flip', action='store_true',
        help='Flip the properties of the near and far path segments')
    args = parser.parse_args()

    keys = DEFAULT_KEYS if len(args.http) == 0 else args.http
    path = f'{args.logdir}/latencies/timeout{args.timeout}/loss{args.loss}p'
    os.system(f'mkdir -p {path}')

    key_data = {}
    for key in keys:
        filename = f'{path}/{key}.txt'
        os.system(f'touch {filename}')
        maybe_collect_missing_data(filename, key, args)
        data, _ = parse_data_cdf(filename, args)
        if data is not None:
            key_data[key] = data
    plot_percentile_vs_latency_graph_flipped(args, key_data,
                                             min_x=args.min_x,
                                             xs=[x / 10.0 for x in range(args.min_x, 1001)],
                                             keys=keys,
                                             pdf=DEFAULT_PDF)
    exit(0)
