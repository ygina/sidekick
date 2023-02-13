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
KEYS = ['cwnd']
bms = ['quack', 'pep_r1', 'quic', 'tcp']
# bms = ['quic', 'quack', 'tcp', 'pep_h2', 'pep_r1']
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

def parse_tcp_data_iperf(filename):
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

def parse_tcp_data_ss(filename):
    with open(filename) as f:
        lines = f.read().split('\n')
    xs = []
    ys = []

    interval_s = 0.1
    for line in lines:
        line = line.strip()
        r = r'.*cwnd:(\d+).*'
        m = re.search(r, line)
        if m is None:
            continue
        m = m.groups()
        y = float(m[0]) * 1.5
        x = interval_s * len(ys)
        xs.append(x)
        ys.append(y)

    return (xs, ys)

def plot_graph(data, iperf, max_x_arg, loss):
    xy = {}
    for bm in ['quic', 'quack']:
        if bm in bms:
            (xs, ys) = parse_quic_data(data[bm])
            xy[bm] = (xs['cwnd'], ys['cwnd'])
    for bm in ['tcp', 'pep_h2', 'pep_r1']:
        if bm in bms:
            if iperf:
                xy[bm] = parse_tcp_data_iperf(data[bm])
            else:
                xy[bm] = parse_tcp_data_ss(data[bm])

    plt.figure(figsize=(9, 6))
    plot_data = []
    for bm in bms:
        if bm in xy:
            (xs, ys) = xy[bm]
            plot_data.append((xs, ys, bm))
    for (xs, ys, label) in plot_data:
        ys = [y / 1.5 for y in ys]
        plt.plot(xs, ys, label=LABEL_MAP[label])

    plt.xlabel('Time (s)')
    plt.ylabel('cwnd (packets)')
    if max_x_arg is not None:
        plt.xlim(0, max_x_arg)
    plt.ylim(0, 240)
    # plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.4), ncol=2)
    pdf = 'cwnd_{}s_loss{}p.pdf'.format(max_x_arg, loss)
    # plt.title(pdf)
    print(pdf)
    save_pdf(pdf)
    plt.clf()

def run(args, loss):
    data = {}
    # basically hardcoded time to get the right-length tcp cwnd test
    if args.tcp is None:
        time_s = int(25 * (float(loss) + 1))
    else:
        time_s = int(args.tcp)
    tcp_test = 'iperf' if args.iperf else 'ss'
    data['quic'] = 'cwnd_quic_{}_loss{}p.out'.format(args.quic_n, loss)
    data['quack'] = 'cwnd_quack_{}_loss{}p.out'.format(args.quack_n, loss)
    data['tcp'] = f'cwnd_tcp_{time_s}s_loss{loss}p_{tcp_test}.out'
    data['pep_h2'] = f'cwnd_pep_h2_{time_s}s_loss{loss}p_{tcp_test}.out'
    data['pep_r1'] = f'cwnd_pep_r1_{time_s}s_loss{loss}p_{tcp_test}.out'

    for key in bms:
        data[key] = f'{WORKDIR}/results/cwnd/{data[key]}'
        filename = data[key]
        print(filename)
        if path.exists(filename):
            continue
        if not args.execute:
            print('ERROR: path does not exist: {}'.format(filename))
            exit(1)

        cmd = ['sudo', '-E', 'python3', 'mininet/net.py', '--loss2', loss]
        if key == 'quic':
            assert args.quic_n is not None
            cmd += ['-n', args.quic_n, '--benchmark', 'quic', '-t', '1']
        elif key == 'quack':
            assert args.quack_n is not None
            cmd += ['-n', args.quack_n, '--benchmark', 'quic', '-t', '1',
                    '-s', '2ms']
        elif args.iperf:
            if key == 'tcp':
                cmd += ['--iperf', str(time_s)]
            elif key == 'pep_h2':
                cmd += ['--iperf', str(time_s), '--pep']
            elif key == 'pep_r1':
                cmd += ['--iperf-r1', str(time_s)]
        else:
            if key == 'tcp':
                cmd += ['--ss', str(time_s), 'h2']
            elif key == 'pep_h2':
                cmd += ['--ss', str(time_s), 'h2', '--pep']
            elif key == 'pep_r1':
                cmd += ['--ss', str(time_s), 'r1', '--pep']

        cmd += args.args
        print(' '.join(cmd))
        p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT)
        with open(filename, 'wb') as f:
            for line in p.stdout:
                f.write(line)
        p.wait()

    plot_graph(data, iperf=args.iperf, max_x_arg=args.max_x, loss=loss)

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
    parser.add_argument('--iperf', action='store_true', help="use iperf instead of ss")
    args = parser.parse_args()

    if args.loss is not None:
        LOSSES = [args.loss]
    for loss in LOSSES:
        run(args, loss)
