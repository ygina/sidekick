import argparse
import subprocess
import os
import sys
import os.path
import statistics
from os import path
from common import *

DATE = None
NUM_TRIALS = None
MAX_X = None
EXECUTE = None
WORKDIR = None
TARGET_XS = [x for x in range(100, 1001, 100)] + \
            [x for x in range(1000, 10001, 1000)] + \
            [x for x in range(10000, 100001, 10000)]

class DataPoint:
    def __init__(self, arr, normalize=None):
        if normalize is not None:
            arr = [normalize * 1. / x for x in arr]
        arr.sort()
        mid = int(len(arr) / 2)
        self.p50 = statistics.median(arr)
        if len(arr) % 2 == 1:
            self.p25 = statistics.median(arr[:mid+1])
        else:
            self.p25 = statistics.median(arr[:mid])
        self.p75 = statistics.median(arr[mid:])
        self.minval = arr[0]
        self.maxval = arr[-1]
        self.avg = statistics.mean(arr)
        self.stdev = None if len(arr) == 1 else statistics.stdev(arr)

def execute_cmd(loss, http_version, cc, trials, data_size):
    suffix = f'loss{loss}p/{cc}'
    results_file = f'{WORKDIR}/results/{suffix}/{http_version}.txt'
    if http_version == 'pep':
        bm = ['tcp', '--pep']
    elif http_version == 'tcp-tso':
        bm = ['tcp', '--tso']
    elif http_version == 'pep-tso':
        bm = ['tcp', '--tso', '--pep']
    elif 'quack' in http_version:
        bm = ['quic', '--sidecar', http_version[6:]]
    elif 'quic-' in http_version:
        bm = ['quic']
    else:
        bm = [http_version]
    subprocess.Popen(['mkdir', '-p', f'results/{suffix}'], cwd=WORKDIR).wait()
    cmd = ['sudo', '-E', 'python3', 'mininet/net.py', '--loss2', str(loss),
        '--benchmark'] + bm + ['-cc', cc, '-t', str(trials),
        '-n', f'{data_size}k']
    print(cmd)
    p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT)
    with open(results_file, 'ab') as f:
        for line in p.stdout:
            sys.stdout.buffer.write(line)
            sys.stdout.buffer.flush()
            f.write(line)
    p.wait()

def parse_data(loss, cc, http_version, normalize, data_key='time_total'):
    """
    Parses the median keyed time and the data size.
    ([data_size], [time_total])
    """
    filename = get_filename(loss, cc, http_version)
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
        if line == '' or '***' in line:
            # Done reading data for this data_size
            if len(data) > 0:
                if data_size not in xy_map:
                    xy_map[data_size] = []
                xy_map[data_size].extend(data)
                if len(xy_map[data_size]) > NUM_TRIALS:
                    # truncated = len(xy_map[data_size]) - NUM_TRIALS
                    # print(f'{data_size}k truncating {truncated} points')
                    xy_map[data_size] = xy_map[data_size][:NUM_TRIALS]
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
        if data_size in TARGET_XS and data_size <= MAX_X:
            xs.append(data_size)
    xs.sort()
    if len(xs) != len(TARGET_XS):
        missing_xs = []
        for x in TARGET_XS:
            if x in xs or x > MAX_X:
                continue
            missing_xs.append(x)
        if EXECUTE:
            for x in missing_xs:
                execute_cmd(loss, http_version, cc, NUM_TRIALS, x)
        elif len(missing_xs) > 0:
            print(f'missing {len(missing_xs)} xs: {missing_xs}')
    try:
        for i in range(len(xs)):
            x = xs[i]
            y = xy_map[x]
            if len(y) < NUM_TRIALS:
                missing = NUM_TRIALS - len(y)
                if EXECUTE:
                    execute_cmd(loss, http_version, cc, missing, x)
                else:
                    print(f'{x}k missing {missing}/{NUM_TRIALS}')
            xs[i] /= 1000.
            ys.append(DataPoint(y, normalize=xs[i] if normalize else None))
    except Exception as e:
        import pdb; pdb.set_trace()
        raise e
    return (xs, ys)

def get_filename(loss, cc, http):
    """
    Args:
    - loss: <number>
    - cc: reno, cubic
    - http: tcp, quic, pep
    """
    return '../results/{}/loss{}p/{}/{}.txt'.format(DATE, loss, cc, http)

def plot_graph(loss, cc, pdf,
               data_key='time_total',
               http_versions=['tcp-tso', 'pep-tso', 'quic'],
               # http_versions=['tcp-tso', 'pep-tso', 'quic', 'quack-152'],
               use_median=True,
               normalize=True):
    data = {}
    for http_version in http_versions:
        filename = get_filename(loss, cc, http_version)
        if not path.exists(filename):
            print('Path does not exist: {}'.format(filename))
            continue
        try:
            data[http_version] = parse_data(loss, cc, http_version, normalize,
                                            data_key=data_key)
        except Exception as e:
            print('Error parsing: {}'.format(filename))
            print(e)
    plt.clf()
    for (i, label) in enumerate(http_versions):
        if label not in data:
            continue
        (xs, ys_raw) = data[label]
        if use_median:
            ys = [y.p50 for y in ys_raw]
            yerr_lower = [y.p50 - y.p25 for y in ys_raw]
            yerr_upper = [y.p75 - y.p50 for y in ys_raw]
            plt.errorbar(xs, ys, yerr=(yerr_lower, yerr_upper), capsize=5,
                label=label, marker=MARKERS[i])
        else:
            ys = [y.avg for y in ys_raw]
            yerr = [y.stdev if y.stdev is not None else 0 for y in ys_raw]
            plt.errorbar(xs, ys, yerr=yerr, label=label, marker=MARKERS[i])
    plt.xlabel('Data Size (MB)')
    if normalize:
        plt.ylabel('{} tput (MB/s)'.format(data_key))
    else:
        plt.ylabel('{} (s)'.format(data_key))
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.4), ncol=3)
    statistic = 'median' if use_median else 'mean'
    plt.title('{} {} {}% loss'.format(statistic, cc, loss))
    if pdf is not None:
        save_pdf(pdf)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--execute',
                        action='store_true',
                        help='Execute benchmarks for missing data points')
    parser.add_argument('-t', '--trials',
                        default=20,
                        type=int,
                        help='Number of trials to plot (default: 20)')
    parser.add_argument('--max-x',
                        default='50000',
                        type=int,
                        help='Maximum x to plot, in kB (default: 40000)')
    parser.add_argument('--mean',
                        action='store_true',
                        help='Plot mean graphs')
    parser.add_argument('--median',
                        action='store_true',
                        help='Plot median graphs')
    parser.add_argument('--loss',
                        type=int,
                        help='Loss percentages to plot [0|1|2|5]. If no '
                             'argument is provided, plots 1, 2, and 5.')
    parser.add_argument('--cc',
                        default='cubic',
                        help='TCP congestion control algorithm to plot '
                             '[reno|cubic] (default: cubic)')
    parser.add_argument('--workdir',
                        default=os.environ['HOME'] + '/sidecar',
                        help='Working directory (default: $HOME/sidecar)')
    parser.add_argument('--date',
                        default='',
                        help='Find results at '
                             '../results/<DATE>/loss<LOSS>p/<CC>/<HTTP>.txt, '
                             'usually something like 010922 if archived '
                             '(default: \'\')')
    args = parser.parse_args()

    if args.loss is None:
        losses = [1, 2, 5]
    else:
        losses = [args.loss]
    cc = args.cc
    NUM_TRIALS = args.trials
    DATE = args.date
    MAX_X = args.max_x
    EXECUTE = args.execute
    WORKDIR = args.workdir
    for loss in losses:
        if args.median:
            plot_graph(loss=loss, cc=cc, pdf='median_{}_loss{}p.pdf'.format(cc, loss), use_median=True)
        if args.mean:
            plot_graph(loss=loss, cc=cc, pdf='mean_{}_loss{}p.pdf'.format(cc, loss), use_median=False)
