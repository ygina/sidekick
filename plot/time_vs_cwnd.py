import argparse
import subprocess
import os
import re
import sys
import os.path
import statistics
from os import path
from common import *

KEYS = ['cwnd']
WORKDIR = os.environ['HOME'] + '/sidecar'

def parse_quic_data(filename):
    with open(filename) as f:
        lines = f.read().split('\n')
    xs = {}
    ys = {}
    events = []
    for key in KEYS:
        xs[key] = []
        ys[key] = []

    for line in lines:
        line = line.strip()
        r = r'^(\S+) (\d+) Instant \{ tv_sec: (\d+), tv_nsec: (\d+) \} \((.*)\)'
        m = re.search(r, line)
        if m is None:
            continue
        m = m.groups()
        key = m[0]
        if key not in KEYS:
            continue
        y = int(m[1]) / 1000.
        x = 1.0 * int(m[2]) + int(m[3]) / 1_000_000_000.
        events.append(m[4]) # The reason for logging the congestion window
        xs[key].append(x)   # Number of seconds according to the Instant
        ys[key].append(y)   # Congestion window, in kB

    if len(xs['cwnd']) == 0:
        return (xs, ys)
    min_x = min([min(xs[key]) for key in KEYS])
    for key in KEYS:
        xs[key] = [x - min_x for x in xs[key]]  # Normalize by initial time

    ###########################################################################
    # The following is some post-processing on events that cause cwnd drops.
    _xs = xs['cwnd']
    _ys = ys['cwnd']
    decreases = []  # The indexes at which the cwnd decreases (due to loss)
    for i in range(len(_ys) - 1):
        if _ys[i] > _ys[i+1]:
            decreases.append(i)
    reasons1 = [events[i] for i in decreases]   # Reason prior to cwnd decrease
    reasons2 = [events[i+1] for i in decreases] # Reason for cwnd decrease
    times2 = [_xs[i+1] for i in decreases]      # Value the cwnd decreases to
    # import pdb; pdb.set_trace()
    # for i in range(len(times2)):
    #     print('{} {}'.format(reasons2[i], times2[i]))
    count = 0  # Number of times the congestion window changes due to quacks
    for i in range(len(_xs)):
        if _xs[i] > 3.0:
            start_i = i
            break
    for i in range(len(_xs)):
        if _xs[i] < 30.0:
            end_i = i
        else:
            break
    for i in range(start_i, end_i):
        if events[i] == 'on_quack_received':
            count += 1
    # print(f'{count} / 13500 = {count/13500.0}')
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

def get_filename(time_s, http, loss):
    filename = f'cwnd_{http}_{time_s}s_loss{loss}p.out'
    directory = f'{WORKDIR}/results/cwnd/'
    os.system(f'mkdir -p {directory}')
    return f'{directory}{filename}'

def parse_data(args, bm, filename, key):
    if not path.exists(filename):
        return ([], [])
    if 'quic' in bm or 'quack' in bm:
        (xs, ys) = parse_quic_data(filename)
        return (xs[key], ys[key])
    if bm in ['tcp', 'pep_h2', 'pep_r1']:
        if args.iperf:
            return parse_tcp_data_iperf(filename)
        else:
            return parse_tcp_data_ss(filename)

def execute_and_parse_data(args, bm, loss, key='cwnd'):
    filename = get_filename(args.time, bm, loss)
    print(filename)
    (xs, ys) = parse_data(args, bm, filename, key)
    if len(xs) > 0 and len(ys) > 0:
        return (xs, ys)
    if not args.execute:
        print(f'ERROR: missing data in {filename}')
        return ([], [])

    cmd =  ['sudo', '-E', 'python3', 'mininet/main.py']
    cmd += ['--loss2', loss]
    cmd += ['--delay1', args.delay1, '--delay2', args.delay2]
    cmd += ['--bw1', args.bw1, '--bw2', args.bw2]
    cmd += ['--threshold', args.threshold]
    if 'quic' in bm:
        cmd += ['-n', f'{args.time}M', '-t', '1']
        cmd += ['--timeout', str(args.time)]
        cmd += ['--min-ack-delay', args.min_ack_delay]
        cmd += ['quic']
    elif 'quack' in bm:
        cmd += ['-n', f'{args.time}M', '-t', '1']
        cmd += ['--timeout', str(args.time)]
        cmd += ['--min-ack-delay', args.min_ack_delay]
        cmd += ['quack']
    elif args.iperf:
        if bm == 'tcp':
            cmd += ['--iperf', str(args.time)]
        elif bm == 'pep_h2':
            cmd += ['--iperf', str(args.time), '--pep']
        elif bm == 'pep_r1':
            cmd += ['--iperf-r1', str(args.time)]
    else:
        if bm == 'tcp':
            cmd += ['monitor', '--ss', str(args.time), 'h2']
        elif bm == 'pep_h2':
            cmd += ['--pep', 'monitor', '--ss', str(args.time), 'h2']
        elif bm == 'pep_r1':
            cmd += ['--pep', 'monitor', '--ss', str(args.time), 'r1']

    cmd += args.args
    print(' '.join(cmd))
    p = subprocess.Popen(cmd, cwd=WORKDIR, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT)
    with open(filename, 'wb') as f:
        for line in p.stdout:
            f.write(line)
    p.wait()
    return parse_data(args, bm, filename, key)

def print_average_cwnd(bm, xs, ys):
    # Bucket the logged cwnds for each second. Take the average of the logged
    # cwnds for a certain second. If there are no logged cwnds, take the average
    # cwnd from the previous second. If the first data point does not have any
    # logged cwnds, set the initial cwnd to 0.
    all_cwnds = [[] for _ in range(int(max(xs))+1)]
    avg_cwnds = []
    for (x, y) in zip(xs, ys):
        all_cwnds[int(x)].append(y)
    for ys in all_cwnds:
        if len(ys) > 0:
            avg_cwnds.append(statistics.mean(ys))
        elif len(avg_cwnds) == 0:
            avg_cwnds.append(0)
        else:
            avg_cwnds.append(avg_cwnds[-1])

    # Skip the first 5 seconds and take the average of all remaining cwnds.
    SECS_TO_SKIP = 5
    if len(avg_cwnds) <= SECS_TO_SKIP:
        avg_cwnd = 0
    else:
        avg_cwnd = statistics.mean(avg_cwnds[SECS_TO_SKIP:])
    print('{}: {}'.format(bm, avg_cwnd))

def run(args, https, loss):
    xy_bm = []
    for bm in https:
        # Execute the benchmark for any data we need to collect.
        (xs, ys) = execute_and_parse_data(args, bm, loss)
        xy_bm.append((xs, ys, bm))
        # Parse bytes_in_flight if flag is set
        if args.bytes_in_flight and ('quic' in bm or 'quack' in bm):
            (xs, ys) = execute_and_parse_data(args, bm, loss, key='bytes_in_flight')
            xy_bm.append((xs, ys, f'{bm}_BIF'))

    plt.figure(figsize=(9, 6))
    for (xs, ys, bm) in xy_bm:
        ys = [y / 1.5 for y in ys]  # Convert kB to packets.
        if bm in LABEL_MAP:
            label = LABEL_MAP[bm]
        else:
            label = bm
        plt.plot(xs, ys, label=label)
        print_average_cwnd(bm, xs, ys)

    plt.xlabel('Time (s)')
    plt.ylabel('cwnd (packets)')
    if args.max_x is not None:
        plt.xlim(0, args.max_x)
    plt.ylim(0, 250)
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.4), ncol=2)
    pdf = 'cwnd_{}s_loss{}p.pdf'.format(args.time, loss)
    plt.title(pdf)
    save_pdf(pdf)
    plt.clf()

if __name__ == '__main__':
    DEFAULT_LOSSES = ['0', '0.25', '1', '2', '5']
    DEFAULT_PROTOCOLS = ['quack', 'pep_h2', 'pep_r1', 'quic', 'tcp']

    parser = argparse.ArgumentParser()
    parser.add_argument('--execute', action='store_true')
    parser.add_argument('--bytes_in_flight', action='store_true')
    parser.add_argument('--time', required=True, type=int, metavar='S',
        help='time to run each experiment, in seconds')
    parser.add_argument('--max-x', type=int, metavar='S', help='max-x axis')
    parser.add_argument('--http', action='extend', nargs='+', default=[],
        help=f'HTTP versions. (default: {DEFAULT_PROTOCOLS})')
    parser.add_argument('--args', action='extend', nargs='+', default=[],
        help='additional arguments to append to the mininet/net.py command if executing.')
    parser.add_argument('--iperf', action='store_true', help="use iperf instead of ss")

    ############################################################################
    # QUIC/QuACK configuration
    quic_config = parser.add_argument_group('quic_config')
    quic_config.add_argument('--min-ack-delay', default='0',
        help='Server minimum ack delay (default: 0)')

    ############################################################################
    # Network configuration
    net_config = parser.add_argument_group('net_config')
    net_config.add_argument('--delay1', default='75', help='(default: 75)')
    net_config.add_argument('--delay2', default='1', help='(default: 1)')
    net_config.add_argument('--threshold', default='100', help=('default: 100'))
    net_config.add_argument('--bw1', default='10', help='(default: 10)')
    net_config.add_argument('--bw2', default='100', help='(default: 100)')
    net_config.add_argument('--loss', action='extend', nargs='+', default=[],
        help=f'loss percentages e.g, 0 (default: {DEFAULT_LOSSES})')

    # Parse arguments
    args = parser.parse_args()
    https = DEFAULT_PROTOCOLS if len(args.http) == 0 else args.http
    losses = DEFAULT_LOSSES if len(args.loss) == 0 else args.loss

    for loss in losses:
        run(args, https, loss)
