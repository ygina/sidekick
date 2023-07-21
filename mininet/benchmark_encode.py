import argparse
import sys
import time
import math
import multiprocessing
from network import *
from mininet.log import setLogLevel

NUM_LINES = 2
NUM_HEADER_BYTES = 14 + 20 + 8

def start_iperf(net, args):
    target_pps = args.tput * args.num_clients
    target_bits = args.tput * args.length * 8;
    sclog(f'Target rate is {target_pps} packets/s ({target_bits / 1000000} * {args.num_clients} Mbit/s)')

    clients = []
    cmd = ['iperf', '-c', '10.0.1.10', '--udp']
    cmd += ['--time', str(args.warmup + args.timeout + 10)]
    cmd += ['-b', str(int(args.tput * args.length * 8))]
    cmd += ['-l', str(args.length)]
    for i in range(args.num_clients):
        core = i % (args.cores - 1)
        new_cmd = ['taskset', '-c', str(core)] + cmd # + ['--cport', f'{5200+i+1}']
        if i < NUM_LINES or i == args.num_clients - 1:
            sclog(' '.join(new_cmd))
        elif i == NUM_LINES:
            sclog('...')
        p = net.h2.popen(new_cmd, stderr=subprocess.STDOUT, stdout=subprocess.PIPE)
        clients.append(p)
    return clients

def print_loadgen_output(clients):
    for i, client in enumerate(clients):
        if i < NUM_LINES or i == args.num_clients - 1:
            sys.stdout.buffer.write(client.stdout.peek())
            sys.stdout.buffer.write(b'\n')
        elif i == NUM_LINES:
            sclog('...')
    sys.stdout.buffer.flush()

def print_sidecar_output(sidecar):
    for line in sidecar.stdout.peek().split(b'\n'):
        if b'DONE' in line:
            break
        sys.stdout.buffer.write(line)
        sys.stdout.buffer.write(b'\n')
        sys.stdout.buffer.flush()

def run_benchmark(net, args, binary):
    """
    1. Set up the network
    2. Start the load generator at the given rate on h2.
        * The load generator collects statistics from the first packet sent
          after the warmup time.
    3. Start the quack sender listening on r1.
        * The quack sender collects statistics from the first packet sniffed.
    4. Wait <timeout> seconds.
    5. Stop the quack sender and load generator. Print statistics.
        * Quack sender: tput (packets/s); latency median, mean, stdev, min,
          max (ns/packet)
        * Load generator: target tput (packets/s); tput (packets/s)
    """
    # load_generator = net.h2.popen(f'./target/release/load_generator --warmup {args.warmup} --tput {args.tput}'.split(' '))
    clients = start_iperf(net, args)
    time.sleep(args.warmup)
    if args.disable_sidecar:
        time.sleep(args.timeout)
        print_loadgen_output(clients)
    else:
        env = os.environ.copy()
        # env['RUST_LOG'] = 'debug'
        benchmark_cmd = f'taskset -c {args.cores - 1} ./target/release/{binary} --threshold {args.threshold} --frequency {args.frequency}'
        sclog(benchmark_cmd)
        r1 = net.r1.popen(benchmark_cmd.split(' '),
            stderr=subprocess.STDOUT, stdout=subprocess.PIPE, env=env)
        time.sleep(args.timeout)
        r1.terminate()
        print_loadgen_output(clients)
        print_sidecar_output(r1)
    target_pps = args.tput * args.num_clients
    if 'multi' not in binary:
        print(f'\nTarget combined rate (packets/s): {round(target_pps, 3)}')
        print(f'Target combined rate (Mbit/s): {round(target_pps * (NUM_HEADER_BYTES + args.length) * 8 / 1000000, 3)}')
    else:
        print(f'\nTarget average rate (packets/s): {round(target_pps / args.num_clients, 3)}')
        print(f'Target average rate (Mbit/s): {round(target_pps * (NUM_HEADER_BYTES + args.length) * 8 / 1000000 / args.num_clients, 3)}')
    net.stop()

if __name__ == '__main__':
    setLogLevel('info')

    parser = argparse.ArgumentParser(prog='Benchmark Encoder')
    subparsers = parser.add_subparsers(required=True)

    ############################################################################
    # Network Configurations
    parser.add_argument('--warmup', '-w', type=int, default=3, metavar='S',
        help='Warmup time, in seconds (default: 3)')
    parser.add_argument('--timeout', '-t', type=int, default=5, metavar='S',
        help='Timeout, in seconds (default: 5)')
    parser.add_argument('--cores', '-c', type=int, default=multiprocessing.cpu_count(),
        help='Number of cores, to partition iperf servers/clients and the '
             'sidecar. On an m5.xlarge, a core can run two clients, one server '
             'generating load at max 100k packets/s, or the sidecar. The last '
            f'core is assigned to the sidecar. (default: {multiprocessing.cpu_count()})')
    parser.add_argument('--disable-sidecar', action='store_true',
        help='Disable the sidecar to test only iperf load generator')

    ############################################################################
    # Load generator configurations
    loadgen_config = parser.add_argument_group('loadgen_config')
    loadgen_config.add_argument('--length', '-l', default=40, type=int, metavar='BYTES',
        help='Target load generator packet length, the -l option in iperf3 '
             '(default: 40)')

    ############################################################################
    # Sidecar configurations
    sidecar_config = parser.add_argument_group('sidecar_config')
    sidecar_config.add_argument('--frequency', type=int, default=0,
        help='Quack frequency in ms (default: 0)')
    sidecar_config.add_argument('--threshold', type=int, default=20, metavar='PACKETS',
        help='Quack threshold (default: 20)')

    ############################################################################
    # Single quack
    single = subparsers.add_parser('single')
    single.set_defaults(binary='benchmark_encode', clients_per_core=1)
    single.add_argument('--tput', default=160000, type=int, metavar='PPS',
        help='Target load generator throughput in packets per second for each '
             'iperf client. The load generator may not be able to achieve too '
             'high of throughputs. (default: 160000)')
    single.add_argument('--num-clients', '-n', default=3, type=int,
        help='Number of iperf clients. (default: 3)')

    ############################################################################
    # Multiple quacks
    #
    # Each client sends at 83.333 packets/s (1 Mbit/s with 1500 byte MTU).
    multi = subparsers.add_parser('multi')
    multi.set_defaults(binary='benchmark_encode_multi')
    multi.add_argument('--tput', default=83.333, type=float, metavar='PPS',
        help='Target load generator throughput in packets per second for each '
             'iperf client. (default: 83.333)')
    multi.add_argument('--num-clients', '-n', default=100, type=int,
        help='Number of iperf clients. (default: 100)')
    multi.add_argument('--clients-per-core', default=1000, type=int,
        help='Number of iperf clients per core (default: 400)')

    args = parser.parse_args()
    os.system('pkill -9 -f iperf')
    os.system('pkill -9 -f ./target/release/benchmark')
    num_client_cores = math.ceil(1.0 * args.num_clients / args.clients_per_core)
    total_num_cores = num_client_cores + 1
    if total_num_cores > args.cores:
        sclog(f'Need {total_num_cores} cores for {args.num_clients} clients')
        exit(1)

    net = SidecarNetwork(delay1=0, delay2=0, loss1=0, loss2=0, bw1=0,
        bw2=0, qdisc='none')
    run_benchmark(net, args, args.binary)
