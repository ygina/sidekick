import argparse
import os
import re
import sys
import subprocess
from common import *

WORKDIR = os.environ['HOME'] + '/sidekick'
BIT_WIDTHS = [16, 32, 63]
BRANCHES = {}
BRANCHES[16] = 'masot-16-bit-precomputed'
BRANCHES[32] = 'main'
BRANCHES[63] = 'masot-montgom-63-bit'
MY_ENV = os.environ.copy()
MY_ENV['RUST_LOG'] = 'info'

def get_filename(bm, width):
    return f'{WORKDIR}/results/bit_widths/{bm}_{width}.txt'

def checkout(width):
    branch = BRANCHES[width]
    subprocess.Popen(['git', 'checkout', branch], cwd=WORKDIR).wait()
    subprocess.Popen(['cargo', 'b', '--release'], cwd=WORKDIR).wait()

class Construct:
    def __init__(self, args):
        self.bm = 'construct'
        self.n = args.n
        self.trials = args.trials
        self.xs = [1] + [x for x in range(5, args.max_x + 1, 5)]

    def execute(self, width):
        filename = get_filename(self.bm, width)
        checkout(width)
        subprocess.Popen(['rm', filename], cwd=WORKDIR).wait()
        for threshold in self.xs:
            cmd = ['./target/release/quack-bm', self.bm,
                   'power-sum', '--trials', str(self.trials),
                   '-n', str(self.n), '-t', str(threshold)]
            p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT, env=MY_ENV)
            with open(filename, 'ab') as f:
                for line in p.stdout:
                    if b'SUMMARY' in line:
                        sys.stdout.buffer.write(line)
                        sys.stdout.buffer.flush()
                    f.write(line)
            p.wait()
        print(filename)

    def parse_data(self):
        data = {}
        for width in BIT_WIDTHS:
            filename = get_filename(self.bm, width)
            ys = []
            with open(filename) as f:
                lines = f.read().split('\n')
            for line in lines:
                line = line.strip()
                if 'SUMMARY' not in line:
                    continue
                m = re.search(r'avg = (\S+)µs', line)
                if m is None:
                    print(line)
                    exit(1)
                us = m.group(1)
                per_packet_ns = float(us) / self.n * 1000
                ys.append(per_packet_ns)
            data[width] = ys
        return data

class Decode:
    def __init__(self, args):
        self.bm = 'decode'
        self.n = args.n
        self.trials = args.trials
        self.threshold = args.threshold
        self.xs = [x for x in range(1, self.threshold+1)]

    def execute(self, width):
        filename = get_filename(self.bm, width)
        checkout(width)
        subprocess.Popen(['rm', filename], cwd=WORKDIR).wait()
        for dropped in self.xs:
            cmd = ['./target/release/quack-bm', self.bm,
                   'power-sum', '--trials', str(self.trials),
                   '-n', str(self.n), '-t', str(self.threshold),
                   '--dropped', str(dropped)]
            p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT, env=MY_ENV)
            with open(filename, 'ab') as f:
                for line in p.stdout:
                    if b'SUMMARY' in line:
                        sys.stdout.buffer.write(line)
                        sys.stdout.buffer.flush()
                    f.write(line)
            p.wait()

    def parse_data(self):
        data = {}
        for width in BIT_WIDTHS:
            filename = get_filename(self.bm, width)
            ys = []
            with open(filename) as f:
                lines = f.read().split('\n')
            for line in lines:
                line = line.strip()
                if 'SUMMARY' not in line:
                    continue
                m = re.search(r'avg = (\S+)µs', line)
                if m is None:
                    print(line)
                    exit(1)
                us = m.group(1)
                ys.append(float(us))
            data[width] = ys
        return data

def plot(bm, xmax, xlabel, ylabel, pdf):
    plt.clf()
    data = bm.parse_data()
    for (i, width) in enumerate(BIT_WIDTHS):
        ys = data[width]
        xs = bm.xs[:len(ys)]
        plt.plot(xs, ys, color=COLOR_MAP[width],
                 marker=MARKERS[i], label=f'{width} bits')
    plt.xlabel(xlabel)
    plt.ylabel(ylabel)
    plt.xlim(0, xmax)
    plt.ylim(0)
    # plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.4), ncol=4)
    # plt.title(pdf)
    if pdf is not None:
        print(pdf)
        save_pdf(pdf)

def run_construct(args, pdf='bit_widths_construct.pdf'):
    bm = Construct(args)
    if args.execute:
        for width in BIT_WIDTHS:
            bm.execute(width)

    xlabel = f'Threshold'
    ylabel = 'Time (ns)'
    plot(bm, args.max_x, xlabel, ylabel, pdf)

    legend_pdf = 'bit_widths_legend.pdf'
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.4), ncol=4, frameon=True)
    bbox = Bbox.from_bounds(-1.2, 4.7, 9, 0.95)
    save_pdf(legend_pdf, bbox_inches=bbox)
    print(legend_pdf)

def run_decode(args, pdf='bit_widths_decode.pdf'):
    bm = Decode(args)
    if args.execute:
        for width in BIT_WIDTHS:
            bm.execute(width)

    xlabel = f'Missing (n={args.n},t={args.threshold})'
    ylabel = 'Time (μs)'
    plot(bm, args.threshold+1, xlabel, ylabel, pdf)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true')
    parser.add_argument('-n', default=1000, type=int,
        help='log size (default: 1000)')
    parser.add_argument('-t', '--trials', default=10, type=int,
        help='number of trials (default: 10)')
    parser.add_argument('--max-x', default=50, type=int,
        help='for construct, maximum threshold, increments of 5 starting at 5 (default: 50)')
    parser.add_argument('--threshold', default=20, type=int,
        help='for decode, threshold (default: 20)')

    args = parser.parse_args()
    run_construct(args)
    run_decode(args)
