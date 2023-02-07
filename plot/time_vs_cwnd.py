import argparse
import subprocess
import os
import re
import sys
import os.path
import statistics
from os import path
from common import *

LOSSES = ['0', '0.25', '1', '2', '5']
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

def plot_graph(tcp_filename, quic_filename, quack_filename, max_x_arg,
               loss):
    xy_quic = parse_quic_data(quic_filename)
    xy_quack = parse_quic_data(quack_filename)
    (xs_tcp, ys_tcp) = parse_tcp_data(tcp_filename)

    max_x = 0
    plt.figure(figsize=(9, 6))
    for (i, key) in enumerate(KEYS):
        if key != 'cwnd':
            continue
        for ((xs, ys), label) in [(xy_quic, 'quic'), (xy_quack, 'quack')]:
            xs = xs[key]
            ys = ys[key]
            plt.plot(xs, ys, label=f'{label}')
            max_x = max(max_x, max(xs))
    plt.plot(xs_tcp, ys_tcp, label='tcp')

    plt.xlabel('Time (s)')
    plt.ylabel('cwnd (kBytes)')
    if max_x_arg is not None:
        plt.xlim(0, max_x_arg)
    else:
        plt.xlim(0, max_x)
    plt.ylim(0, 600)
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.3), ncol=3)
    pdf = 'cwnd_{}s_loss{}p.pdf'.format(max_x_arg, loss)
    plt.title(pdf)
    print(pdf)
    save_pdf(pdf)
    plt.clf()

def run(args, loss):
    quic_filename = 'cwnd_quic_{}_loss{}p.out'.format(args.quic_n, loss)
    quic_filename = f'{WORKDIR}/results/cwnd/{quic_filename}'
    print(quic_filename)
    if not path.exists(quic_filename):
        if not args.execute:
            print('ERROR: path does not exist: {}'.format(quic_filename))
            exit(1)
        assert args.quic_n is not None
        cmd = ['sudo', '-E', 'python3', 'mininet/net.py', '-n', args.quic_n,
               '--loss2', loss, '-t', '1', '--benchmark', 'quic']
        cmd += args.args
        print(' '.join(cmd))
        p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT)
        with open(quic_filename, 'wb') as f:
            for line in p.stdout:
                f.write(line)
        p.wait()

    quack_filename = 'cwnd_quack_{}_loss{}p.out'.format(args.quack_n, loss)
    quack_filename = f'{WORKDIR}/results/cwnd/{quack_filename}'
    print(quack_filename)
    if not path.exists(quack_filename):
        if not args.execute:
            print('ERROR: path does not exist: {}'.format(quack_filename))
            exit(1)
        assert args.quack_n is not None
        cmd = ['sudo', '-E', 'python3', 'mininet/net.py', '-n', args.quack_n,
               '--loss2', loss, '-t', '1', '--benchmark', 'quic', '-s', '2ms']
        cmd += args.args
        print(' '.join(cmd))
        p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT)
        with open(quack_filename, 'wb') as f:
            for line in p.stdout:
                f.write(line)
        p.wait()

    # basically hardcoded time to get the right-length iperf test
    if args.tcp is None:
        time_s = int(25 * (float(loss) + 1))
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
        quack_filename=quack_filename, max_x_arg=args.max_x, loss=loss)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true')
    parser.add_argument('--quic-n', required=True, help='quic data size')
    parser.add_argument('--quack-n', required=True, help='quack data size')
    parser.add_argument('--tcp', required=True,
                        help='how long to run the tcp iperf test, in s')
    parser.add_argument('--loss', help='loss perecentage e.g, 0')
    parser.add_argument('--max-x', type=float, help='max x axis')
    parser.add_argument('--args', action='extend', nargs='+', default=[],
        help='additional arguments to append to the mininet/net.py command if executing.')
    args = parser.parse_args()

    if args.loss is not None:
        LOSSES = [args.loss]
    for loss in LOSSES:
        run(args, loss)
