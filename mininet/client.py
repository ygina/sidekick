import argparse
import os
import tempfile
from common import sclog

def print_and_run_cmd(cmd):
    sclog(cmd)
    return os.system(cmd)

def run_client(args, base_command, http_flag):
    f = tempfile.NamedTemporaryFile()
    print_and_run_cmd(f'head -c {args.n} /dev/urandom > {f.name}')
    print(f'Data Size: {args.n}')
    print(f'HTTP: {http_flag}')

    cmd =  f'{base_command} {http_flag} --insecure '
    cmd += f'--data-binary @{f.name} '
    cmd += f'https://{args.addr}/ '
    if args.trials is None:
        sclog(cmd)
        fmt="\n\n      time_connect:  %{time_connect}s\n   time_appconnect:  %{time_appconnect}s\ntime_starttransfer:  %{time_starttransfer}s\n                   ----------\n        time_total:  %{time_total}s\n\nexitcode: %{exitcode}\nresponse_code: %{response_code}\nsize_upload: %{size_upload}\nsize_download: %{size_download}\nerrormsg: %{errormsg}\n"
        cmd += f'-w \"{fmt}\" '
        os.system(f'eval \'{cmd}\'')
    else:
        fmt="%{time_connect}\\t%{time_appconnect}\\t%{time_starttransfer}\\t\\t%{time_total}\\t%{exitcode}\\t\\t%{response_code}\\t\\t%{size_upload}\\t\\t%{size_download}\\t%{errormsg}\\n"
        cmd += f'--max-time {args.timeout} '
        cmd += f'-o {args.stdout} 2>>{args.stderr} '
        sclog(cmd)
        cmd += f'-w \"{fmt}\" '
        header = 'time_connect\ttime_appconnect\ttime_starttransfer\ttime_total\texitcode\tresponse_code\tsize_upload\tsize_download\terrormsg'
        print(header)
        for _ in range(args.trials):
            os.system(f'eval \'{cmd}\'')

def run_tcp_client(args):
    cmd = 'RUST_LOG=debug sidecurl '
    run_client(args, cmd, '--http1.1')

def run_quic_client(args):
    cmd = ''
    if args.qlog:
        cmd += 'QLOGDIR=/home/gina/sidekick/qlog '
    cmd += 'RUST_LOG=debug RUST_BACKTRACE=1 '
    if args.quack_style == 'strawman_c':
        cmd += '/home/gina/sidekick/http3_integration/curl/sidecurl/tcpsidecurl '
    else:
        cmd += 'sidecurl '
    cmd += f'--sidekick {args.threshold} '
    cmd += f'--enable-reset {int(args.quack_reset)} '
    if args.quack_style:
        cmd += f'--quack-style {args.quack_style} '
    if args.disable_mtu_fix:
        cmd += '--disable-mtu-fix '
    if args.min_ack_delay is not None:
        cmd += f'--min-ack-delay {args.min_ack_delay} '
    if args.max_ack_delay is not None:
        cmd += f'--max-ack-delay {args.max_ack_delay} '
    if args.mark_acked is not None:
        cmd += f'--mark-acked {int(args.mark_acked)} '
    if args.mark_lost_and_retx is not None:
        cmd += f'--mark-lost-and-retx {int(args.mark_lost_and_retx)} '
    if args.update_cwnd is not None:
        cmd += f'--update-cwnd {int(args.update_cwnd)} '
    if args.near_delay is not None:
        cmd += f'--near-delay {args.near_delay} '
    if args.e2e_delay is not None:
        cmd += f'--e2e-delay {args.e2e_delay} '
    if args.reset_port is not None:
        cmd += f'--reset-port {args.reset_port} '
    if args.reset_threshold is not None:
        cmd += f'--reset-threshold {args.reset_threshold} '
    if args.reorder_threshold is not None:
        cmd += f'--reorder-threshold {args.reorder_threshold} '
    run_client(args, cmd, '--http3')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(prog='Sidecar Client')
    parser.add_argument('-n',
                        required=True,
                        help='Number of bytes to send e.g. 1M')
    parser.add_argument('-t', '--trials', type=int,
                        help='Number of trials')
    parser.add_argument('--stdout',
                        default='/dev/null',
                        metavar='FILE',
                        help='File to write stdout to (default: /dev/null)')
    parser.add_argument('--stderr',
                        default='/dev/null',
                        metavar='FILE',
                        help='File to write stderr to (default: /dev/null)')
    parser.add_argument('--addr',
                        default='10.0.1.10:443',
                        help='Server address (default: 10.0.1.10:443)')
    parser.add_argument('--timeout', type=int,
                        help='Timeout, in seconds (default: None).')

    subparsers = parser.add_subparsers(required=True)
    tcp = subparsers.add_parser('tcp')
    tcp.set_defaults(func=run_tcp_client)
    quic = subparsers.add_parser('quic')
    quic.add_argument('--threshold', type=int, default=0,
                      help='The quACK threshold. (default: 0)')
    quic.add_argument('--min-ack-delay', type=int, default=0, metavar='MS',
                      help='Min delay between acks. (default: 0)')
    quic.add_argument('--max-ack-delay', type=int, default=25, metavar='MS',
                      help='Max delay between acks. (default: 25)')
    quic.add_argument('--disable-mtu-fix', action='store_true',
                      help='Disable fix that sends packets only if cwnd > mtu')
    quic.add_argument('--quack-reset', type=bool, default=True,
                      help='Whether to send quack reset messages [0|1] (default: 1)')
    quic.add_argument('--quack-style', default='power_sum',
                      help='Style of quack to send/receive (default: power_sum)',
                      choices=['power_sum', 'strawman_a', 'strawman_b', 'strawman_c'])
    quic.add_argument('--qlog', action='store_true',
                      help='Store qlogs at $HOME/sidekick/qlog')
    quic.add_argument('--mark-acked', type=bool)
    quic.add_argument('--mark-lost-and-retx', type=bool)
    quic.add_argument('--update-cwnd', type=bool)
    quic.add_argument('--near-delay', type=int, metavar='MS')
    quic.add_argument('--e2e-delay', type=int, metavar='MS')
    quic.add_argument('--reset-port', type=int)
    quic.add_argument('--reset-threshold', type=int, metavar='MS')
    quic.add_argument('--reorder-threshold', type=int, metavar='PKTS')
    quic.set_defaults(func=run_quic_client)
    args = parser.parse_args()

    args.func(args)
