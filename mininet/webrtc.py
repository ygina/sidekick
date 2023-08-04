import argparse
import sys
import time
import subprocess
from common import *
from network import *
from mininet.log import setLogLevel

def start_webrtc_server(net, args):
    cmd = './target/release/webrtc_server '
    cmd += '--port 5123 --client-addr 10.0.2.10:5124 '
    cmd += f'-b {args.client_bytes} '
    cmd += f'--rtt {2 * (args.delay1 + args.delay2)} '
    print(cmd)
    cmd = cmd.strip().split(' ')
    return net.h1.popen(cmd, stderr=subprocess.STDOUT, stdout=subprocess.PIPE)

def start_webrtc_client(net, args):
    cmd = './target/release/webrtc_client '
    cmd += '--server-addr 10.0.1.10:5123 --port 5124 '
    cmd += f'--timeout {args.timeout} '
    cmd += f'-b {args.client_bytes} '
    cmd += f'-f {args.client_frequency} '
    if args.sidecar:
        cmd += f'--quack-port 5103 '
        cmd += f'--quack-style {args.style} --threshold {args.threshold} '
    print(cmd)
    cmd = cmd.strip().split(' ')
    return net.h2.popen(cmd, stderr=subprocess.STDOUT, stdout=subprocess.PIPE)

def flush_process(p):
    for line in p.stdout:
        sys.stdout.buffer.write(line)
        sys.stdout.buffer.flush()

def benchmark(net, args):
    server = start_webrtc_server(net, args)
    time.sleep(1)
    client = start_webrtc_client(net, args)
    print(f'client exitcode = {client.wait()}')
    print(f'server exitcode = {server.wait()}')
    flush_process(client)
    flush_process(server)

if __name__ == '__main__':
    setLogLevel('info')

    parser = argparse.ArgumentParser(prog='Sidecar')
    subparsers = parser.add_subparsers(required=True)

    ############################################################################
    # Network Configurations
    net_config = parser.add_argument_group('net_config')
    net_config.add_argument('--delay1', type=int, default=75, metavar='MS',
        help='1/2 RTT between h1 and r1 (default: 75)')
    net_config.add_argument('--delay2', type=int, default=1, metavar='MS',
        help='1/2 RTT between r1 and h2 (default: 1)')
    net_config.add_argument('--loss1', type=int, default=0, metavar='PERCENT',
        help='loss (in %%) between h1 and r1 (default: 0)')
    net_config.add_argument('--loss2', type=str, default='1', metavar='PERCENT',
        help='loss (in %%) between r1 and h2 (default: 1)')
    net_config.add_argument('--bw1', type=int, default=10, metavar='MBPS',
        help='link bandwidth (in Mbps) between h1 and r1 (default: 10)')
    net_config.add_argument('--bw2', type=int, default=100, metavar='MBPS',
        help='link bandwidth (in Mbps) between r1 and h2 (default: 100)')
    net_config.add_argument('--qdisc', default='grenville',
        help='queuing discipline [tbf|cake|codel|red|grenville|none]')

    ############################################################################
    # Client configurations
    client_config = parser.add_argument_group('client_config')
    client_config.add_argument('-b', '--client-bytes', type=int, default=240,
        help='Number of bytes of dummy data in the payload (default: 240)')
    client_config.add_argument('-f', '--client-frequency', type=int, default=50,
        help='Frequency to send packets, in milliseconds (default: 50)')
    client_config.add_argument('--timeout', type=int, metavar='S', default=30,
        help='Timeout, in seconds (default: 30)')

    ############################################################################
    # Baseline benchmark
    base = subparsers.add_parser('base')
    base.set_defaults(sidecar=False)

    ############################################################################
    # QuACK benchmark
    quack = subparsers.add_parser('quack')
    quack.set_defaults(sidecar=True)
    quack.add_argument('--style', default='power_sum',
        choices=['power_sum', 'strawman_a', 'strawman_b', 'strawman_c'])
    quack.add_argument('--frequency', default='10ms',
        help='Quack frequency, in terms of ms or packets e.g., 2ms or 2p '
             '(default: 10ms)')
    quack.add_argument('--threshold', type=int, default=5,
        metavar='PACKETS',
        help='Initializes the quACK sender and receiver with this threshold '
             '(default: 5).')

    args = parser.parse_args()
    net = SidecarNetwork(args.delay1, args.delay2, args.loss1, args.loss2,
        args.bw1, args.bw2, args.qdisc)
    if args.sidecar:
        net.start_quack_sender(args.frequency, args.threshold, args.style)
    clean_logs()

    benchmark(net, args)
    net.stop()
