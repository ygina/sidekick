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
        h2_cmd += f' --max-ack-delay {args.client_max_ack_delay} '
        h2_cmd += f' --sidecar-mtu {int(args.sidecar_mtu)} '
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

    # Add sidecar-specific flags
    h2_cmd += f'quic '
    h2_cmd += f'--min-ack-delay {args.client_min_ack_delay} '
    h2_cmd += f'--max-ack-delay {args.client_max_ack_delay} '
    h2_cmd += f'--sidecar-mtu {int(args.sidecar_mtu)} '
    h2_cmd += f'--threshold {args.threshold} '
    h2_cmd += f'--quack-reset {int(args.quack_reset)} '

    net.h2.cmdPrint(h2_cmd)
    for _ in range(loops):
        net.start_quack_sender(args.frequency, args.threshold)
        time.sleep(0.1)  # wait for the quack sender to start
        net.h2.cmdPrint(h2_cmd)


if __name__ == '__main__':
    setLogLevel('info')

    parser = argparse.ArgumentParser(prog='Sidecar')
    subparsers = parser.add_subparsers(required=True)
    cli = subparsers.add_parser('cli')
    cli.set_defaults(ty='cli')

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
    proto_config.add_argument('-s', '--sidecar', action='store_true',
        help='Enables the sidecar and sends the quack with the specified '
             'frequency.')
    proto_config.add_argument('--frequency', default='2ms',
        help='Quack frequency, in terms of ms or packets e.g., 2ms or 2p '
             '(default: 2ms)')
    proto_config.add_argument('--threshold', type=int, default=20,
        metavar='PACKETS',
        help='Initializes the quACK sender and receiver with this threshold '
             '(default: 20).')

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
    quic.set_defaults(ty='benchmark', benchmark=benchmark_quic, sidecar=False)
    quic.add_argument('--client-min-ack-delay', type=int, default=0, metavar='MS',
        help='Minimum delay between acks, in ms (default: 0)')
    quic.add_argument('--client-max-ack-delay', type=int, default=25, metavar='MS',
        help='Maximum delay between acks, in ms (default: 25)')
    quic.add_argument('--sidecar-mtu', type=bool, default=True,
        metavar='ENABLED',
        help='Send packets only if cwnd > mtu [0|1] (default: 1)')

    ############################################################################
    # QuACK client benchmark
    quack = subparsers.add_parser('quack')
    quack.set_defaults(ty='benchmark', benchmark=benchmark_quack, sidecar=True)
    quack.add_argument('--client-min-ack-delay', type=int, default=0, metavar='MS',
        help='Minimum delay between acks, in ms (default: 0)')
    quack.add_argument('--client-max-ack-delay', type=int, default=25, metavar='MS',
        help='Maximum delay between acks, in ms (default: 25)')
    quack.add_argument('--sidecar-mtu', type=bool, default=True,
        metavar='ENABLED',
        help='Send packets only if cwnd > mtu [0|1] (default: 1)')
    quack.add_argument('--quack-reset', type=bool, default=True,
        metavar='ENABLED',
        help='Whether to send quack reset messages [0|1] (default: 1)')

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
    network_monitor = parser.add_argument_group('network_monitor')
    group = network_monitor.add_mutually_exclusive_group()
    group.add_argument('--iperf-r1', type=int, metavar='TIME_S',
        help='Run an iperf test for this length of time with a server on h1 '
             'and client on r1.')
    group.add_argument('--iperf', type=int, metavar='TIME_S',
        help='Run an iperf test for this length of time with a server on h1 '
             'and client on h2.')
    network_monitor.add_argument('--ping', type=int,
        help='Run this many pings from h2 to h1.')
    network_monitor.add_argument('--ss', nargs=2, metavar=('TIME_S', 'HOST'),
        help='Run an ss test for this length of time, in s (while uploading a '
             'a 100M file) on this host. Gets ss data every 0.1s of a TCP '
             'TCP connection to h1.')

    args = parser.parse_args()
    net = SidecarNetwork(args.delay1, args.delay2, args.loss1, args.loss2,
        args.bw1, args.bw2, args.qdisc, args.min_ack_delay, args.max_ack_delay)
    sys.stderr.buffer.write(bytes(f'Link1 delay={args.delay1} loss={args.loss1} bw={args.bw1}\n', 'utf-8'))
    sys.stderr.buffer.write(bytes(f'Link2 delay={args.delay2} loss={args.loss2} bw={args.bw2}\n', 'utf-8'))
    sys.stderr.buffer.flush()
    if args.pep:
        net.start_tcp_pep()
    if args.sidecar:
        net.start_quack_sender(args.frequency, args.threshold)
    net.set_segmentation_offloading(args.tso)
    clean_logs()

    if args.ping is not None:
        run_ping(net, args.ping)
    elif args.ss is not None:
        run_ss(net, int(args.ss[0]), args.ss[1])
    elif args.iperf is not None:
        run_iperf(sc, args.iperf, host='h2')
    elif args.iperf_r1 is not None:
        run_iperf(sc, args.iperf_r1, host='r1')
    elif args.ty == 'multiflow':
        assert not args.pep and not args.sidecar
        run_multiflow(sc, args.flow1, args.flow2, args.delay)
    elif args.ty == 'cli':
        CLI(net.net)
    else:
        tx1 = net.get_h1_tx_packets()
        args.benchmark(net, args)
        tx2 = net.get_h1_tx_packets()
        print(f'h1-eth0 tx_packets = {tx2 - tx1}')
    net.stop()
