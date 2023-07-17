import argparse
import sys
import time
import math
import multiprocessing
from network import *
from mininet.log import setLogLevel

NUM_LINES = 5

def start_iperf_servers(net, args):
    servers = []
    num_server_cores = math.ceil(1.0 * args.num_clients / args.servers_per_core)
    for i in range(args.num_clients):
        cmd = f'taskset -c {int(i % num_server_cores)} iperf3 -s -f m -p {5200+i+1}'.split(' ')
        if i < NUM_LINES or i == args.num_clients - 1:
            sclog(' '.join(cmd))
        elif i == NUM_LINES:
            sclog('...')
        p = net.h1.popen(cmd, stderr=subprocess.STDOUT, stdout=subprocess.PIPE)
        servers.append(p)
    return servers

def start_iperf_clients(net, args):
    target_pps = args.tput * args.num_clients
    target_bits = args.tput * args.length * 8;
    sclog(f'Target rate is {target_pps} packets/s ({target_bits / 1000000} * {args.num_clients} Mbit/s)')

    clients = []
    cmd = ['iperf3', '-c', '10.0.1.10', '--udp', '--congestion', 'cubic']
    cmd += ['--time', str(args.warmup + args.timeout + 1)]
    cmd += ['-b', str(int(args.tput * args.length * 8))]
    cmd += ['-l', str(args.length)]
    num_client_cores = math.ceil(1.0 * args.num_clients / args.clients_per_core)
    num_server_cores = math.ceil(1.0 * args.num_clients / args.servers_per_core)
    for i in range(args.num_clients):
        core = num_server_cores + (i % num_client_cores)
        new_cmd = ['taskset', '-c', str(core)] + cmd + ['-p', f'{5200+i+1}']
        if i < NUM_LINES or i == args.num_clients - 1:
            sclog(' '.join(new_cmd))
        elif i == NUM_LINES:
            sclog('...')
        p = net.h2.popen(new_cmd, stderr=subprocess.STDOUT, stdout=subprocess.PIPE)
        clients.append(p)
    return clients

def start_iperf(net, args):
    servers = start_iperf_servers(net, args)
    time.sleep(1)
    clients = start_iperf_clients(net, args)
    return (servers, clients)

def print_loadgen_output(servers, clients):
    for i, server in enumerate(servers):
        if i < NUM_LINES or i == args.num_clients - 1:
            sys.stdout.buffer.write(server.stdout.peek())
            sys.stdout.buffer.write(b'\n')
        elif i == NUM_LINES:
            sclog('...')
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
    servers, clients = start_iperf(net, args)
    time.sleep(args.warmup)
    if args.disable_sidecar:
        time.sleep(args.timeout)
        print_loadgen_output(servers, clients)
    else:
        env = os.environ.copy()
        # env['RUST_LOG'] = 'debug'
        benchmark_cmd = f'taskset -c {args.cores - 1} ./target/release/{binary} --threshold {args.threshold} --frequency {args.frequency}'
        sclog(benchmark_cmd)
        r1 = net.r1.popen(benchmark_cmd.split(' '),
            stderr=subprocess.STDOUT, stdout=subprocess.PIPE, env=env)
        time.sleep(args.timeout)
        r1.terminate()
        print_loadgen_output(servers, clients)
        print_sidecar_output(r1)
    target_pps = args.tput * args.num_clients
    print(f'Target combined rate (packets/s): {round(target_pps, 3)}')
    print(f'Target combined rate (Mbit/s): {round(target_pps * 1500 * 8 / 1000000, 3)}')
    print(f'Target average rate (packets/s): {round(target_pps / args.num_clients, 3)}')
    print(f'Target average rate (Mbit/s): {round(target_pps * 1500 * 8 / 1000000 / args.num_clients, 3)}')
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
    loadgen_config.add_argument('--length', '-l', default=70, type=int, metavar='BYTES',
        help='Target load generator packet length, the -l option in iperf3 '
             '(default: 70)')

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
    single.set_defaults(binary='benchmark_encode')
    single.add_argument('--tput', default=80000, type=int, metavar='PPS',
        help='Target load generator throughput in packets per second for each '
             'iperf client. The load generator may not be able to achieve too '
             'high of throughputs. (default: 80000)')
    single.add_argument('--num-clients', '-n', default=2, type=int,
        help='Number of iperf clients. (default: 2)')
    single.add_argument('--clients-per-core', default=1, type=int,
        help='Number of iperf clients per core (default: 1)')
    single.add_argument('--servers-per-core', default=2, type=int,
        help='Number of iperf servers per core (default: 2)')

    ############################################################################
    # Multiple quacks
    #
    # Each client sends at 83.333 packets/s (1 Mbit/s with 1500 byte MTU).
    multi = subparsers.add_parser('multi')
    multi.set_defaults(binary='benchmark_encode_multi')
    multi.add_argument('--tput', default=83, type=int, metavar='PPS',
        help='Target load generator throughput in packets per second for each '
             'iperf client. (default: 83)')
    multi.add_argument('--num-clients', '-n', default=100, type=int,
        help='Number of iperf clients. (default: 100)')
    multi.add_argument('--clients-per-core', default=1000, type=int,
        help='Number of iperf clients per core (default: 1000)')
    multi.add_argument('--servers-per-core', default=2000, type=int,
        help='Number of iperf servers per core (default: 2000)')

    args = parser.parse_args()
    num_client_cores = math.ceil(1.0 * args.num_clients / args.clients_per_core)
    num_server_cores = math.ceil(1.0 * args.num_clients / args.servers_per_core)
    total_num_cores = num_client_cores + num_server_cores + 1
    if total_num_cores > args.cores:
        sclog(f'Need {total_num_cores} cores for {args.num_clients} clients')
        exit(1)

    net = SidecarNetwork(delay1=0, delay2=0, loss1=0, loss2=0, bw1=0,
        bw2=0, qdisc='none')
    run_benchmark(net, args, args.binary)
