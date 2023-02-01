import argparse
import subprocess
import os
import re
import sys
import os.path
import statistics
from os import path
from common import *

LOSSES = ['0', '1', '2']
KEYS = ['cwnd', 'bytes_in_flight']
WORKDIR = os.environ['HOME'] + '/sidecar'

def parse_quic_data(filename):
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
        y = int(m[1]) / 1000.
        x = 1.0 * int(m[2]) + int(m[3]) / 1_000_000_000.
        xs[key].append(x)
        ys[key].append(y)

    min_x = min([min(xs[key]) for key in KEYS])
    for key in KEYS:
        xs[key] = [x - min_x for x in xs[key]]
    return (xs, ys)

def parse_tcp_data(filename):
    with open(filename) as f:
        lines = f.read().split('\n')
    xs = []
    ys = []

    for line in lines:
        line = line.strip()
        r = r'.*\]\s+(\S+)-.*\s(\S+) KBytes$'
        m = re.search(r, line)
        if m is None:
            continue
        m = m.groups()
        y = float(m[1])
        x = float(m[0])
        xs.append(x)
        ys.append(y)

    return (xs, ys)

def plot_graph(tcp_filename, quic_filename, data_size, loss):
    (xs_quic, ys_quic) = parse_quic_data(quic_filename)
    (xs_tcp, ys_tcp) = parse_tcp_data(tcp_filename)

    max_x = 0
    for (i, key) in enumerate(KEYS):
        if key != 'cwnd':
            continue
        xs = xs_quic[key]
        ys = ys_quic[key]
        plt.plot(xs, ys, label=f'quic_{key}')
        max_x = max(max_x, max(xs))
    plt.plot(xs_tcp, ys_tcp, label='tcp_cwnd')

    plt.xlabel('Time (s)')
    plt.ylabel('kBytes')
    plt.xlim(0, max_x)
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.4), ncol=2)
    pdf = 'cwnd_{}_loss{}p.pdf'.format(data_size, loss)
    plt.title(pdf)
    print(pdf)
    save_pdf(pdf)
    plt.clf()

def run(args, loss):
    quic_filename = 'cwnd_quic_{}_loss{}p.out'.format(args.n, loss)
    quic_filename = f'{WORKDIR}/results/cwnd/{quic_filename}'
    print(quic_filename)
    if not path.exists(quic_filename):
        if not args.execute:
            print('ERROR: path does not exist: {}'.format(quic_filename))
            exit(1)
        assert args.n is not None
        cmd = ['sudo', '-E', 'python3', 'mininet/net.py', '-n', args.n,
               '--loss2', loss, '-t', '1', '--benchmark', 'quic']
        cmd += args.args
        print(' '.join(cmd))
        p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT)
        with open(quic_filename, 'wb') as f:
            for line in p.stdout:
                f.write(line)
        p.wait()

    # basically hardcoded time to get the right-length iperf test
    if args.tcp is None:
        time_s = 25 * (int(loss) + 1)
    else:
        time_s = int(args.tcp)
    tcp_filename = 'cwnd_tcp_{}s_loss{}p.out'.format(time_s, loss)
    tcp_filename = f'{WORKDIR}/results/cwnd/{tcp_filename}'
    print(tcp_filename)
    if not path.exists(tcp_filename):
        if not args.execute:
            print('ERROR: path does not exist: {}'.format(tcp_filename))
            exit(1)
        cmd = ['sudo', '-E', 'python3', 'mininet/net.py',
               '--loss2', loss, '--iperf', str(time_s)]
        print(' '.join(cmd))
        p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT)
        with open(tcp_filename, 'wb') as f:
            for line in p.stdout:
                f.write(line)
        p.wait()

    plot_graph(tcp_filename=tcp_filename, quic_filename=quic_filename,
        data_size=args.n, loss=loss)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true')
    parser.add_argument('-n', help='data size e.g., 10M')
    parser.add_argument('--tcp', help='how long to run the tcp iperf test, in s')
    parser.add_argument('--loss', help='loss perecentage e.g, 0')
    parser.add_argument('--args', action='extend', nargs='+', default=[],
        help='additional arguments to append to the mininet/net.py command if executing.')
    args = parser.parse_args()

    if args.loss is not None:
        LOSSES = [args.loss]
    for loss in LOSSES:
        run(args, loss)
