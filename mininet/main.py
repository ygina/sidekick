import argparse
import sys
import time
from common import *
from network import *
from mininet.cli import CLI
from mininet.log import setLogLevel


def benchmark(net, args, proxy, quic, client):
    if args.timeout:
        timeout = args.timeout
    else:
        timeout = estimate_timeout(args.n, proxy, quic, loss=args.loss2)
    h2_cmd = f'python3 mininet/client.py -n {args.n} ' \
             f'--stdout {args.stdout} --stderr {args.stderr} ' \
             f'--timeout {timeout} '
    if args.trials: h2_cmd += f'-t {args.trials} '
    h2_cmd += f'{client}'
    if quic:
        h2_cmd += f' --min-ack-delay {args.client_min_ack_delay} '
        h2_cmd += f' --max-ack-delay {max(args.client_max_ack_delay, args.min_ack_delay)} '
        if args.disable_mtu_fix:
            h2_cmd += ' --disable-mtu-fix '
    net.h2.cmdPrint(h2_cmd)

def benchmark_tcp(net, args):
    benchmark(net, args, proxy=False, quic=False, client='tcp')

def benchmark_pep(net, args):
    benchmark(net, args, proxy=True, quic=False, client='tcp')

def benchmark_quic(net, args):
    benchmark(net, args, proxy=False, quic=True, client='quic')

def benchmark_quack(net, args):
    if args.timeout:
        timeout = args.timeout
    else:
        timeout = estimate_timeout(n=args.n, proxy=True, quic=True, loss=args.loss2)
    h2_cmd = f'python3 mininet/client.py -n {args.n} ' \
             f'--stdout {args.stdout} --stderr {args.stderr} ' \
             f'--timeout {timeout} '
    if args.trials is None:
        loops = 0
    else:
        loops = args.trials - 1
        h2_cmd += '-t 1 '

    # Add sidekick-specific flags
    h2_cmd += f'quic '
    h2_cmd += f'--min-ack-delay {args.client_min_ack_delay} '
    h2_cmd += f'--max-ack-delay {max(args.client_max_ack_delay, args.min_ack_delay)} '
    h2_cmd += f'--threshold {args.threshold} '
    h2_cmd += f'--quack-reset {int(args.quack_reset)} '
    h2_cmd += f'--quack-style {args.style} '
    h2_cmd += f'--near-delay {args.delay2} '
    h2_cmd += f'--e2e-delay {args.delay1 + args.delay2} '
    h2_cmd += f'--reset-threshold {args.delay2 * 10} '  # 10x near delay
    if args.disable_mtu_fix:
        h2_cmd += '--disable-mtu-fix '
    if args.mark_acked is not None:
        h2_cmd += f'--mark-acked {int(args.mark_acked)} '
    if args.mark_lost_and_retx is not None:
        h2_cmd += f'--mark-lost-and-retx {int(args.mark_lost_and_retx)} '
    if args.update_cwnd is not None:
        h2_cmd += f'--update-cwnd {int(args.update_cwnd)} '
    if args.reset_port is not None:
        h2_cmd += f'--reset-port {args.reset_port} '
    if args.reorder_threshold is not None:
        h2_cmd += f'--reorder-threshold {args.reorder_threshold} '

    net.h2.cmdPrint(h2_cmd)
    for _ in range(loops):
        net.start_quack_sender(args.frequency, args.threshold, args.style,
                               quack_sender_host=self.r1,
                               quack_sender_iface='r1-eth1',
                               quack_sender_ipaddr='10.0.2.1',
                               quack_receiver_sockaddr='10.0.2.10:5103')
        time.sleep(0.1)  # wait for the quack sender to start
        net.h2.cmdPrint(h2_cmd)


if __name__ == '__main__':
    setLogLevel('info')

    parser = argparse.ArgumentParser(prog='Sidekick')
    subparsers = parser.add_subparsers(required=True)
    cli = subparsers.add_parser('cli')
    cli.set_defaults(ty='cli')

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
    net_config.add_argument('--min-ack-delay', type=int, default=0, metavar='MS',
        help='minimum delay between ACKs from the webserver (default: 0)')
    net_config.add_argument('--max-ack-delay', type=int, default=25, metavar='MS',
        help='maximum delay between ACKs from the webserver (default: 25)')

    ############################################################################
    # TCP/QUIC-Specific Network Configurations
    proto_config = parser.add_argument_group('proto_config')
    proto_config.add_argument('-p', '--pep', action='store_true',
        help='Start a TCP pep on r1')
    proto_config.add_argument('--tso', type=bool, default=True,
        metavar='ENABLED',
        help='Enable TCP segment offloading (tso) and generic '
             'segment offloading (gso). By default, both are '
             'enabled [0|1] (default: 1)')
    proto_config.add_argument('-s', '--sidekick', action='store_true',
        help='Enables the sidekick and sends the quack with the specified '
             'frequency. Sends quacks from r1 to h2.')
    proto_config.add_argument('-b', '--buffer', action='store_true',
        help='Enables the buffering sidekick and sends the quack with the'
             'specified frequency. Sends quacks from h1 to r1.')
    proto_config.add_argument('--frequency', default='30ms',
        help='Quack frequency, in terms of ms or packets e.g., 2ms or 2p '
             '(default: 30ms)')
    proto_config.add_argument('--threshold', type=int, default=10,
        metavar='PACKETS',
        help='Initializes the quACK sender and receiver with this threshold '
             '(default: 10).')
    proto_config.add_argument('--style', default='power_sum',
        choices=['power_sum', 'strawman_a', 'strawman_b', 'strawman_c'])
    proto_config.add_argument('--print-statistics', action='store_true',
        help='Print statistics on number of packets sent at each host')

    ############################################################################
    # Client configurations
    client_config = parser.add_argument_group('client_config')
    client_config.add_argument('-n', default='1M', metavar='BYTES_STR',
        help='Number of bytes (default: 1M)')
    client_config.add_argument('-t', '--trials', type=int,
        help='Number of trials')
    client_config.add_argument('--stdout', default='/tmp/scstdout', metavar='FILENAME',
        help='File to write curl stdout (default: /tmp/scstdout)')
    client_config.add_argument('--stderr', default='/tmp/scstderr', metavar='FILENAME',
        help='File to write curl stderr (default: /tmp/scstderr)')
    client_config.add_argument('--timeout', type=int, metavar='S',
        help='Timeout, in seconds. Default is estimated.')

    ############################################################################
    # TCP client benchmark
    tcp = subparsers.add_parser('tcp')
    tcp.set_defaults(ty='benchmark', benchmark=benchmark_tcp, pep=False)

    ############################################################################
    # PEP client benchmark
    pep = subparsers.add_parser('pep')
    pep.set_defaults(ty='benchmark', benchmark=benchmark_pep, pep=True)

    ############################################################################
    # QUIC client benchmark
    quic = subparsers.add_parser('quic')
    quic.set_defaults(ty='benchmark', benchmark=benchmark_quic, sidekick=False)
    quic.add_argument('--client-min-ack-delay', type=int, default=0, metavar='MS',
        help='Minimum delay between acks, in ms (default: 0)')
    quic.add_argument('--client-max-ack-delay', type=int, default=25, metavar='MS',
        help='Maximum delay between acks, in ms (default: 25 or the '
             'min_ack_delay, whichever is larger)')
    quic.add_argument('--disable-mtu-fix', action='store_true',
        help='Disable fix that sends packets only if cwnd > mtu')

    ############################################################################
    # QuACK client benchmark
    quack = subparsers.add_parser('quack')
    quack.set_defaults(ty='benchmark', benchmark=benchmark_quack, sidekick=True)
    quack.add_argument('--client-min-ack-delay', type=int, default=0, metavar='MS',
        help='Minimum delay between acks, in ms (default: 0)')
    quack.add_argument('--client-max-ack-delay', type=int, default=25, metavar='MS',
        help='Maximum delay between acks, in ms (default: 25 or the server\'s '
             'min_ack_delay, whichever is larger)')
    quack.add_argument('--disable-mtu-fix', action='store_true',
        help='Disable fix that sends packets only if cwnd > mtu')
    quack.add_argument('--quack-reset', type=bool, default=True,
        metavar='ENABLED',
        help='Whether to send quack reset messages [0|1] (default: 1)')
    quack.add_argument('--mark-acked', type=bool)
    quack.add_argument('--mark-lost-and-retx', type=bool)
    quack.add_argument('--update-cwnd', type=bool)
    quack.add_argument('--reset-port', type=int)
    quack.add_argument('--reorder-threshold', type=int, metavar='PKTS')

    ############################################################################
    # Multiflow experiments
    mf = subparsers.add_parser('multiflow', help='run two flows simultaneously')
    mf.set_defaults(ty='multiflow')
    mf.add_argument('-f1', '--flow1', required=True,
                    choices=['quack', 'quic', 'tcp', 'pep'])
    mf.add_argument('-f2', '--flow2', required=True,
                    choices=['quack', 'quic', 'tcp', 'pep'])
    mf.add_argument('-d', '--delay', default=0, type=int,
                    help='delay in starting flow2, in s (default: 0)')

    ############################################################################
    # Network monitoring tests
    monitor = subparsers.add_parser('monitor', help='network monitoring')
    monitor.set_defaults(ty='monitor')
    group = monitor.add_mutually_exclusive_group()
    group.add_argument('--iperf-r1', type=int, metavar='TIME_S',
        help='Run an iperf test for this length of time with a server on h1 '
             'and client on r1.')
    group.add_argument('--iperf', type=int, metavar='TIME_S',
        help='Run an iperf test for this length of time with a server on h1 '
             'and client on h2.')
    monitor.add_argument('--ping', type=int,
        help='Run this many pings from h2 to h1.')
    monitor.add_argument('--ss', nargs=2, metavar=('TIME_S', 'HOST'),
        help='Run an ss test for this length of time, in s (while uploading a '
             'a 100M file) on this host. Gets ss data every 0.1s of a TCP '
             'TCP connection to h1.')

    args = parser.parse_args()
    net = SidekickNetwork(args.delay1, args.delay2, args.loss1, args.loss2,
        args.bw1, args.bw2, args.qdisc)
    net.start_webserver(args.min_ack_delay, args.max_ack_delay)
    sclog(f'Link1 delay={args.delay1} loss={args.loss1} bw={args.bw1}')
    sclog(f'Link2 delay={args.delay2} loss={args.loss2} bw={args.bw2}')
    if args.pep:
        net.start_tcp_pep()
    if args.sidekick:
        net.start_quack_sender(args.frequency, args.threshold, args.style,
                               quack_sender_host=net.r1,
                               quack_sender_iface='r1-eth1',
                               quack_sender_ipaddr='10.0.2.1',
                               quack_receiver_sockaddr='10.0.2.10:5103')
    if args.buffer:
        net.start_quack_sender(args.frequency, args.threshold, args.style,
                               quack_sender_host=net.h1,
                               quack_sender_iface='h1-eth0',
                               quack_sender_ipaddr='10.0.1.10',
                               quack_receiver_sockaddr='10.0.1.1:5103')
    net.set_segmentation_offloading(args.tso)
    net.disable_checksum_offloading()
    clean_logs()

    if args.ty == 'monitor':
        if args.ping is not None:
            run_ping(net, args.ping)
        elif args.ss is not None:
            run_ss(net, int(args.ss[0]), args.ss[1])
        elif args.iperf is not None:
            run_iperf(net, args.iperf, host='h2')
        elif args.iperf_r1 is not None:
            run_iperf(net, args.iperf_r1, host='r1')
    elif args.ty == 'multiflow':
        assert not args.pep and not args.sidekick
        run_multiflow(net, args, args.flow1, args.flow2, args.delay)
    elif args.ty == 'cli':
        CLI(net.net)
    else:
        if args.print_statistics:
            net.statistics.start()
            args.benchmark(net, args)
            net.statistics.stop_and_print()
        else:
            args.benchmark(net, args)
    net.stop()
