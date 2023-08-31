import argparse
import subprocess
import os
import re
import sys
import os.path
import statistics
import dpkt

from datetime import datetime
from collections import defaultdict
from os import path
from common import *

WORKDIR = os.environ['HOME'] + '/sidecar'
GRANULARITY = None
X_MAX = None

def plot_graph(filename, xs, ys0, ys1, f1, f2, pdf):
    plt.figure(figsize=(9, 6))
    plt.plot(xs, ys0, label=LABEL_MAP[f1], color=COLOR_MAP[f1])
    plt.plot(xs, ys1, label=LABEL_MAP[f2], color=COLOR_MAP[f2])
    plt.xlabel('Time (s)')
    plt.ylabel('Throughput (MBytes/s)')
    if X_MAX is not None:
        plt.xlim(0, X_MAX)
    else:
        plt.xlim(0, max(xs))
    plt.ylim(0, 1.4)
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.3), ncol=2)
    plt.title(pdf)
    print(pdf)
    save_pdf(f'{WORKDIR}/plot/graphs/{pdf}')
    plt.clf()

def get_pcap_filename(loss, n, f1, f2, delay, bw):
    prefix = f'{WORKDIR}/results/multiflow/loss{loss}p'
    os.system(f'mkdir -p {prefix}')
    f = f'{prefix}/{f1}_{f2}_{n}_delay{delay}s_bw{bw}.pcap'
    return f

def get_pdf_filename(loss, n, f1, f2, delay, bw):
    f = f'{f1}_{f2}_{n}_loss{loss}p_delay{delay}s_bw{bw}.pdf'
    return f

def execute_cmd(loss, n, f1, f2, delay, bw, timeout):
    cmd = ['sudo', '-E', 'python3', 'mininet/main.py', '-n', n,
       '--loss2', str(loss), '--bw2', str(bw), '--timeout', str(timeout),
       'multiflow', '-f1', f1, '-f2', f2, '-d', str(delay)]
    print(' '.join(cmd))
    p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT)
    for line in p.stdout:
        sys.stdout.buffer.write(line)
        sys.stdout.buffer.flush()

def zero():
    return 0

def parse_pcap(filename):
    data = [defaultdict(zero), defaultdict(zero)]
    sport_to_flow = {}

    epoch = datetime(1970, 1, 1)
    for ts, pkt in dpkt.pcap.Reader(open(filename, 'rb')):
        eth = dpkt.ethernet.Ethernet(pkt)
        assert eth.type == dpkt.ethernet.ETH_TYPE_IP
        ip = eth.data
        sport = ip.data.sport
        if sport not in sport_to_flow:
            sport_to_flow[sport] = len(sport_to_flow)
        ts = datetime.utcfromtimestamp(ts)

        flow = sport_to_flow[sport]
        time_ms = int((ts - epoch).total_seconds() * GRANULARITY)
        nbytes = len(pkt)
        data[flow][time_ms] += nbytes

    xs = []
    ys0 = []
    ys1 = []
    min_x = min([x for x in data[0]] + [x for x in data[1]])
    max_x = max([x for x in data[0]] + [x for x in data[1]])
    for x in range(min_x, max_x+1):
        xs.append((x - min_x) / GRANULARITY)
        ys0.append(data[0][x]/1000000.*GRANULARITY)
        ys1.append(data[1][x]/1000000.*GRANULARITY)

    return (xs, ys0, ys1)

def run(execute, loss, n, f1, f2, delay, bw, timeout):
    filename = get_pcap_filename(loss, n, f1, f2, delay, bw)
    if not path.exists(filename):
        if execute:
            execute_cmd(loss, n, f1, f2, delay, bw, timeout)
        else:
            print('file does not exist:', filename)
            return
    print(filename)
    (xs, ys0, ys1) = parse_pcap(filename)
    pdf = get_pdf_filename(loss, n, f1, f2, delay, bw)
    plot_graph(filename, xs, ys0, ys1, f1, f2, pdf)

def main(args):
    assert args.flow1 is not None
    assert args.flow2 is not None
    run(args.execute, args.loss, args.n, args.flow1, args.flow2, args.delay, args.bw, timeout=args.max_x)

def run_loss0p(args):
    # 30M for every 30 seconds
    if args.n is None:
        n = str(int(args.max_x / 30. * 30)) + 'M'
    else:
        n = args.n
    for delay in [0, 5]:
        run(args.execute, 0, n, 'quic', 'quic', delay, args.bw, timeout=args.max_x)
        run(args.execute, 0, n, 'quic', 'quack', delay, args.bw, timeout=args.max_x)
        run(args.execute, 0, n, 'quack', 'quic', delay, args.bw, timeout=args.max_x)

def run_loss1p(args):
    # 30M for every 30 seconds
    if args.n is None:
        n = str(int(args.max_x / 30. * 30)) + 'M'
    else:
        n = args.n
    for delay in [0, 5]:
        run(args.execute, 1, n, 'pep', 'pep', delay, args.bw, timeout=args.max_x)
        run(args.execute, 1, n, 'pep', 'quack', delay, args.bw, timeout=args.max_x)
        run(args.execute, 1, n, 'quack', 'pep', delay, args.bw, timeout=args.max_x)

def run_loss5p(args):
    # 30M for every 30 seconds
    if args.n is None:
        n = str(int(args.max_x / 30. * 30)) + 'M'
    else:
        n = args.n
    for delay in [0, 5]:
        run(args.execute, 5, n, 'pep', 'pep', delay, args.bw)
        run(args.execute, 5, n, 'pep', 'quack', delay, args.bw)
        run(args.execute, 5, n, 'quack', 'pep', delay, args.bw)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true')
    parser.add_argument('--bw', type=int, default=100,
                        help='bandwidth on near subpath in Mbps (default: 100)')
    parser.add_argument('-n', help='data size (e.g. 10M)')
    parser.add_argument('-g', '--granularity', default=1, type=int,
                        help='1000 for ms, 1 for s, so on (default: 1)')
    parser.add_argument('-f1', '--flow1', help='[quack|quic|tcp|pep]')
    parser.add_argument('-f2', '--flow2', help='[quack|quic|tcp|pep]')
    parser.add_argument('-d', '--delay', default=0, type=int,
                        help='delay in starting flow2, in s (default: 0)')
    parser.add_argument('--max-x', type=int,
                        help='max x, in s')
    parser.add_argument('--loss', default=0, type=int,
                        help='loss on near subpath in %% (default: 0)')
    parser.set_defaults(func=main)

    subparsers = parser.add_subparsers(title='subcommands')
    loss0p = subparsers.add_parser('loss0p', help='quic+quic vs quic+quack')
    loss1p = subparsers.add_parser('loss1p', help='quic+quic vs quic+quack')
    loss5p = subparsers.add_parser('loss5p', help='pep+pep vs pep+quack')
    loss0p.set_defaults(func=run_loss0p)
    loss1p.set_defaults(func=run_loss1p)
    loss5p.set_defaults(func=run_loss5p)

    args = parser.parse_args()
    GRANULARITY = args.granularity
    X_MAX = args.max_x

    args.func(args)
