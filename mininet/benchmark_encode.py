import argparse
import sys
import time
from network import *
from mininet.log import setLogLevel

def start_iperf_servers(net, args):
    servers = []
    for i in range(args.num_clients):
        cmd = f'taskset -c 0 iperf3 -s -f m -p 520{i+1}'.split(' ')
        sclog(' '.join(cmd))
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
    for i in range(args.num_clients):
        new_cmd = ['taskset', '-c', str(i+1)] + cmd + ['-p', f'520{i+1}']
        sclog(' '.join(new_cmd))
        p = net.h2.popen(new_cmd, stderr=subprocess.STDOUT, stdout=subprocess.PIPE)
        clients.append(p)
    return clients

def start_iperf(net, args):
    servers = start_iperf_servers(net, args)
    time.sleep(1)
    clients = start_iperf_clients(net, args)
    return (servers, clients)

def print_loadgen_output(servers, clients):
    for server in servers:
        sys.stdout.buffer.write(server.stdout.peek())
        sys.stdout.buffer.write(b'\n')
    for client in clients:
        sys.stdout.buffer.write(client.stdout.peek())
        sys.stdout.buffer.write(b'\n')
    sys.stdout.buffer.flush()

def print_sidecar_output(sidecar):
    for line in sidecar.stdout.peek().split(b'\n'):
        if b'DONE' in line:
            break
        sys.stdout.buffer.write(line)
        sys.stdout.buffer.write(b'\n')
        sys.stdout.buffer.flush()

def run_benchmark_single(net, args):
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
        # env['RUST_LOG'] = 'trace'
        r1 = net.r1.popen(f'taskset -c 3 ./target/release/benchmark_encode --threshold {args.threshold} --frequency {args.frequency}'.split(' '),
            stderr=subprocess.STDOUT, stdout=subprocess.PIPE, env=env)
        time.sleep(args.timeout)
        r1.terminate()
        print_loadgen_output(servers, clients)
        print_sidecar_output(r1)
    target_pps = args.tput * args.num_clients
    print(f'Target rate (packets/s): {round(target_pps, 3)}')
    print(f'Target rate (Mbit/s): {round(target_pps * 1500 * 8 / 1000000, 3)}')
    net.stop()

def run_benchmark_multi(net, args):
    pass

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
    parser.add_argument('--disable-sidecar', action='store_true',
        help='Disable the sidecar to test only iperf load generator')

    ############################################################################
    # Load generator configurations
    loadgen_config = parser.add_argument_group('loadgen_config')
    loadgen_config.add_argument('--tput', default=50000, type=int, metavar='PPS',
        help='Target load generator throughput in packets per second for each '
             'iperf client. The load generator may not be able to achieve too '
             'high of throughputs. (default: 50000)')
    loadgen_config.add_argument('--length', '-l', default=70, type=int, metavar='BYTES',
        help='Target load generator packet length, the -l option in iperf3 '
             '(default: 70)')
    loadgen_config.add_argument('--num-clients', '-n', default=2, type=int,
        help='Number of iperf clients. (default: 2)')

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
    single.set_defaults(benchmark=run_benchmark_single)

    ############################################################################
    # Multiple quacks
    multi = subparsers.add_parser('multi')
    multi.set_defaults(benchmark=run_benchmark_multi)

    args = parser.parse_args()
    net = SidecarNetwork(delay1=0, delay2=0, loss1=0, loss2=0, bw1=0,
        bw2=0, qdisc='none')
    args.benchmark(net, args)
