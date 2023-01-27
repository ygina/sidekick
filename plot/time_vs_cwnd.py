import argparse
import subprocess
import os
import re
import sys
import os.path
import statistics
from os import path
from common import *

KEYS = ['cwnd', 'bytes_in_flight']
WORKDIR = os.environ['HOME'] + '/sidecar'

def parse_data(filename):
    with open(filename) as f:
        lines = f.read().split('\n')
    xs = {}
    ys = {}
    for key in KEYS:
        xs[key] = []
        ys[key] = []

    for line in lines:
        line = line.strip()
        r = r'^(\S+) (\d+) Instant \{ tv_sec: (\d+), tv_nsec: (\d+) \} .*'
        m = re.search(r, line)
        if m is None:
            continue
        m = m.groups()
        key = m[0]
        assert key in KEYS
        y = int(m[1])
        x = 1.0 * int(m[2]) + int(m[3]) / 1_000_000_000.
        xs[key].append(x)
        ys[key].append(y)

    min_x = min([min(xs[key]) for key in KEYS])
    for key in KEYS:
        xs[key] = [x - min_x for x in xs[key]]
    return (xs, ys)

def plot_graph(filename, data_size, http, loss):
    if not path.exists(filename):
        print('Path does not exist: {}'.format(filename))
        return
    (xs_all, ys_all) = parse_data(filename)

    for (i, key) in enumerate(KEYS):
        xs = xs_all[key]
        ys = ys_all[key]
        plt.errorbar(xs, ys, label=key)
    plt.xlabel('Time (s)')
    plt.ylabel('Bytes')
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.4), ncol=3)
    pdf = 'cwnd_{}_{}_loss{}p.pdf'.format(http, data_size, loss)
    plt.title(pdf)
    print(pdf)
    save_pdf(pdf)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true')
    parser.add_argument('-n', help='data size e.g., 10M')
    parser.add_argument('--http', help='http version e.g., quic')
    parser.add_argument('--loss', help='loss perecentage e.g, 0')
    parser.add_argument('--args', action='extend', nargs='+', default=[],
        help='additional arguments to append to the mininet/net.py command if executing.')
    args = parser.parse_args()

    filename = 'cwnd_{}_{}_loss{}p.out'.format(args.http, args.n, args.loss)
    filename = f'{WORKDIR}/results/cwnd/{filename}'
    print(filename)
    if not path.exists(filename):
        if not args.execute:
            print('ERROR: path does not exist')
            exit(1)
        assert args.n is not None
        assert args.loss is not None
        assert args.http is not None
        cmd = ['sudo', '-E', 'python3', 'mininet/net.py', '-n', args.n,
               '--loss2', args.loss, '-t', '1', '--benchmark', args.http]
        cmd += args.args
        print(' '.join(cmd))
        p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT)
        with open(filename, 'wb') as f:
            for line in p.stdout:
                f.write(line)
        p.wait()
    plot_graph(filename=filename, data_size=args.n, http=args.http,
        loss=args.loss)
