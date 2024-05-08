import argparse
import os
import sys
import time
import subprocess
from common import *
from network import *
from mininet.cli import CLI
from mininet.log import setLogLevel

def start_media_server(net, args, env):
    cmd = './target/release/media_server --port 5123 '
    cmd += f'-b {args.client_bytes} '
    cmd += f'--rtt {2 * (args.delay1 + args.delay2)} '
    print(cmd)
    cmd = cmd.strip().split(' ')
    return net.h1.popen(cmd, stderr=subprocess.STDOUT, stdout=subprocess.PIPE, env=env)

def start_media_client(net, args, env):
    cmd = './target/release/media_client --server-addr 10.0.1.10:5123 '
    cmd += f'--timeout {args.timeout} '
    cmd += f'-b {args.client_bytes} '
    cmd += f'-f {args.client_frequency} '
    if args.sidekick:
        cmd += f'--reset-addr 10.0.2.1:1234 '
        cmd += f'--quack-port 5103 '
        cmd += f'--quack-style {args.style} --threshold {args.threshold} '
    print(cmd)
    cmd = cmd.strip().split(' ')
    return net.h2.popen(cmd, stderr=subprocess.STDOUT, stdout=subprocess.PIPE, env=env)

def flush_process(p):
    for line in p.stdout:
        sys.stdout.buffer.write(line)
        sys.stdout.buffer.flush()

def benchmark(net, args):
    env = os.environ.copy()
    env['RUST_LOG'] = 'quack=info,mio=info,debug'
    env['RUST_BACKTRACE'] = '1'
    server = start_media_server(net, args, env)
    time.sleep(1)
    client = start_media_client(net, args, env)
    print(f'client exitcode = {client.wait()}')
    print(f'server exitcode = {server.wait()}')
    flush_process(client)
    flush_process(server)

if __name__ == '__main__':
    setLogLevel('info')

    parser = argparse.ArgumentParser(prog='Sidekick')
    subparsers = parser.add_subparsers(required=True)
    cli = subparsers.add_parser('cli')
    cli.set_defaults(cli=True)

    ############################################################################
    # Network Configurations
    net_config = parser.add_argument_group('net_config')
    net_config.add_argument('--delay1', type=int, default=25, metavar='MS',
        help='1/2 RTT between h1 and r1 (default: 25)')
    net_config.add_argument('--delay2', type=int, default=1, metavar='MS',
        help='1/2 RTT between r1 and h2 (default: 1)')
    net_config.add_argument('--loss1', type=str, default='0', metavar='PERCENT',
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
    # Sidekick Configurations
    sc_config = parser.add_argument_group('sc_config')
    sc_config.add_argument('--sidekick', action='store_true',
        help='Enable the sidekick')
    sc_config.add_argument('--style', default='power_sum',
        choices=['power_sum', 'strawman_a', 'strawman_b', 'strawman_c'])
    sc_config.add_argument('--frequency', default='10ms',
        help='Quack frequency, in terms of ms or packets e.g., 2ms or 2p '
             '(default: 10ms)')
    sc_config.add_argument('--threshold', type=int, default=5,
        metavar='PACKETS',
        help='Initializes the quACK sender and receiver with this threshold '
             '(default: 5).')

    ############################################################################
    # Client configurations
    client_config = parser.add_argument_group('client_config')
    client_config.add_argument('-b', '--client-bytes', type=int, default=240,
        help='Number of bytes of dummy data in the payload (default: 240)')
    client_config.add_argument('-f', '--client-frequency', type=int, default=20,
        help='Frequency to send packets, in milliseconds (default: 20)')
    client_config.add_argument('--timeout', type=int, metavar='S', default=30,
        help='Timeout, in seconds (default: 30)')

    ############################################################################
    # Baseline benchmark
    base = subparsers.add_parser('base')
    base.set_defaults(sidekick=False, cli=False, buffering=False)

    ############################################################################
    # QuACK benchmark
    quack = subparsers.add_parser('quack')
    quack.set_defaults(sidekick=True, cli=False, buffering=False)

    ############################################################################
    # QuACK benchmark with buffering
    quack = subparsers.add_parser('quack_buffer')
    quack.set_defaults(sidekick=False, cli=False, buffering=True)

    args = parser.parse_args()
    net = SidekickNetwork(args.delay1, args.delay2, args.loss1, args.loss2,
        args.bw1, args.bw2, args.qdisc)
    if args.sidekick:
        net.start_quack_sender(args.frequency, args.threshold, args.style,
                               quack_sender_host=self.r1,
                               quack_sender_iface='r1-eth1',
                               quack_sender_ipaddr='10.0.2.1',
                               quack_receiver_sockaddr='10.0.2.10:5103')
    if args.buffering:
        net.start_buffering_proxy()
        net.start_quack_sender(args.frequency, args.threshold, args.style,
                               quack_sender_host=net.h1,
                               quack_sender_iface='h1-eth0',
                               quack_sender_ipaddr='10.0.1.10',
                               quack_receiver_sockaddr='10.0.1.1:5103')
    clean_logs()

    if args.cli:
        CLI(net.net)
    else:
        benchmark(net, args)
    net.stop()
