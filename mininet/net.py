import argparse
import logging
import sys
import time
import client
import re
import os
from mininet.net import Mininet
from mininet.cli import CLI
from mininet.log import setLogLevel
from mininet.link import TCLink

SLEEP_S = 0.1

def mac(digit):
    assert 0 <= digit < 10
    return f'00:00:00:00:00:0{int(digit)}'

def ip(digit):
    assert 0 <= digit < 10
    return f'10.0.{int(digit)}.10/24'

def sclog(val):
    print(f'[sidecar] {val}', file=sys.stderr);

def get_max_queue_size(rtt_ms, bw_mbps):
    """
    Calculate the maximum queue size as
    Bandwidth Delay Product (BDP) / MTU * 1.1 packets.
    """
    bdp = rtt_ms * bw_mbps * 1000000. / 1000. / 8.
    mtu = 1500
    return int(bdp / mtu * 1.1) + 1

class SidecarNetwork():
    def __init__(self, args):
        self.net=None
        self.pep = args.pep
        self.sidecar = args.sidecar
        self.threshold = args.threshold
        self.delay1 = args.delay1
        self.delay2 = args.delay2
        self.loss1 = args.loss1
        self.loss2 = args.loss2
        self.bw1 = args.bw1
        self.bw2 = args.bw2
        self.log_level = args.log_level
        if args.pep and args.sidecar is not None:
            sclog('only one of the PEP or sidecar can be enabled')
            exit()
        if args.cc not in ['reno', 'cubic']:
            sclog(f'invalid congestion control algorithm: {args.cc}')
            exit()
        self.cc = args.cc
        self.tso = args.tso

    def clean_logs(self):
        os.system('rm -f r1.log h1.log h2.log')

    def start_webserver(self):
        # Start the webserver on h1
        # TODO: not user-dependent path
        sclog('Starting the NGINX/Python webserver on h1...')
        self.h1.cmd("kill $(pidof nginx)")
        self.h1.cmd("nginx -c /home/gina/sidecar/webserver/nginx.conf")
        self.h1.cmd("python3 webserver/server.py >> h1.log 2>&1 &")

    def start_tcp_pep(self):
        # Start the TCP PEP on r1
        sclog('Starting the TCP PEP on r1...')
        self.r1.cmd('ip rule add fwmark 1 lookup 100')
        self.r1.cmd('ip route add local 0.0.0.0/0 dev lo table 100')
        self.r1.cmd('iptables -t mangle -F')
        self.r1.cmd('iptables -t mangle -A PREROUTING -i r1-eth1 -p tcp -j TPROXY --on-port 5000 --tproxy-mark 1')
        self.r1.cmd('iptables -t mangle -A PREROUTING -i r1-eth0 -p tcp -j TPROXY --on-port 5000 --tproxy-mark 1')
        self.r1.cmd('pepsal -v >> r1.log 2>&1 &')

    def start_quack_sender(self):
        # Start the quACK sender on r1
        print('', file=sys.stderr)
        sclog('Starting the QUIC sidecar sender on r1...')
        assert self.sidecar is not None
        if 'ms' in self.sidecar:
            frequency = re.match(r'(\d+)ms', self.sidecar).group(1)
            frequency = f'--frequency-ms {frequency}'
        elif 'p' in self.sidecar:
            frequency = re.match(r'(\d+)p.*', self.sidecar).group(1)
            frequency = f'--frequency-pkts {frequency}'
        else:
            raise 'Invalid frequency: {}'.format(self.sidecar)

        self.r1.cmd(f'kill $(pidof sidecar)')
        self.r1.cmd(f'RUST_BACKTRACE=1 RUST_LOG={self.log_level} ' \
            f'./target/release/sidecar -i r1-eth1 -t {self.threshold} ' + \
            f'quack-sender --target-addr 10.0.2.10:5103 ' + \
            f'{frequency} >> r1.log 2>&1 &')

    def start_and_configure(self):
        self.net = Mininet(controller=None, link=TCLink)

        # Add hosts and switches
        self.h1 = self.net.addHost('h1', ip=ip(1), mac=mac(1))
        self.h2 = self.net.addHost('h2', ip=ip(2), mac=mac(2))
        self.r1 = self.net.addHost('r1')

        # Add links
        rtt_ms = 2 * (self.delay1 + self.delay2)
        bw_mbps = min(self.bw1, self.bw2)
        mqs = get_max_queue_size(rtt_ms, bw_mbps)
        print(f'max_queue_size = {mqs} packets')
        self.net.addLink(self.r1, self.h1,
                         bw=self.bw1,
                         loss=self.loss1,
                         delay=f'{self.delay1}ms',
                         max_queue_size=mqs)
        self.net.addLink(self.r1, self.h2,
                         bw=self.bw2,
                         delay=f'{self.delay2}ms',
                         loss=self.loss2,
                         max_queue_size=mqs)
        self.net.build()

        # Configure interfaces
        self.r1.cmd("ifconfig r1-eth0 0")
        self.r1.cmd("ifconfig r1-eth1 0")
        self.r1.cmd("ifconfig r1-eth0 hw ether 00:00:00:00:01:01")
        self.r1.cmd("ifconfig r1-eth1 hw ether 00:00:00:00:01:02")
        self.r1.cmd("ip addr add 10.0.1.1/24 brd + dev r1-eth0")
        self.r1.cmd("ip addr add 10.0.2.1/24 brd + dev r1-eth1")
        self.r1.cmd("echo 1 > /proc/sys/net/ipv4/ip_forward")
        self.h1.cmd("ip route add default via 10.0.1.1")
        self.h2.cmd("ip route add default via 10.0.2.1")

        # Configure link latency and delay
        # self.h1.cmd(f'tc qdisc add dev h1-eth0 root netem delay {self.delay1}ms')
        # self.h2.cmd(f'tc qdisc add dev h2-eth0 root netem loss {self.loss2}% delay {self.delay2}ms')
        # self.r1.cmd(f'tc qdisc add dev r1-eth0 root netem delay {self.delay1}ms')
        # self.r1.cmd(f'tc qdisc add dev r1-eth1 root netem delay {self.delay2}ms')

        # Set the TCP congestion control algorithm
        sclog(f'Setting congestion control to {self.cc}')
        cc_cmd = f'sysctl -w net.ipv4.tcp_congestion_control={self.cc}'
        self.h1.cmd(cc_cmd)
        self.r1.cmd(cc_cmd)
        self.h2.cmd(cc_cmd)

        # Don't cache TCP metrics
        tcp_metrics_cmd = 'echo 1 > /proc/sys/net/ipv4/tcp_no_metrics_save'
        self.h1.cmd(tcp_metrics_cmd)
        self.r1.cmd(tcp_metrics_cmd)
        self.h2.cmd(tcp_metrics_cmd)

        # Turn off tso and gso to send MTU-sized packets
        sclog('tso and gso are {}'.format('ON' if self.tso else 'OFF'))
        if not self.tso:
            self.h1.cmd('ethtool -K h1-eth0 gso off tso off')
            self.h2.cmd('ethtool -K h2-eth0 gso off tso off')
            self.r1.cmd('ethtool -K r1-eth0 gso off tso off')
            self.r1.cmd('ethtool -K r1-eth1 gso off tso off')

        self.start_webserver()
        if self.pep:
            self.start_tcp_pep()
        elif self.sidecar is not None:
            self.start_quack_sender()
        else:
            sclog('NOT starting the TCP PEP or sidecar')

    def iperf(self, time_s):
        self.start_and_configure()
        self.h1.cmd('iperf3 -s -f m > /dev/null 2>&1 &')
        self.h2.cmdPrint(f'iperf3 -c 10.0.1.10 -t {time_s} -f m -b 20M -C cubic -i 0.1')

    def benchmark(self, nbytes, http_version, trials, cc, stdout_file,
                  stderr_file, quack_log):
        """
        Args:
        - nbytes: Number of bytes to send e.g., 1M.
        - http_version:
            HTTP/1.1 - http/1.1 1.1 1 h1 tcp
            HTTP/3.3 - http/3 3 h3 quic
        - trials
        """
        if http_version is None:
            sclog(f'must set http version: {http_version}')
            return
        http_version = http_version.lower()
        if http_version in ['http/1.1', '1.1', '1', 'h1', 'tcp']:
            http_version = 1
        elif http_version in ['http/3', '3', 'h3', 'quic']:
            http_version = 3
        else:
            sclog(f'must set http version: {http_version}')
            return

        h2_cmd = f'python3 mininet/client.py -n {nbytes} ' \
                 f'--http {http_version} ' \
                 f'--stdout {stdout_file} --stderr {stderr_file} ' \
                 f'-cc {self.cc} --loss {self.loss2} ' \
                 f'--log-level {self.log_level} '
        if self.sidecar is not None:
            h2_cmd += f'--sidecar h2-eth0 {self.threshold} '
        if trials is not None:
            h2_cmd += f'-t {trials} '
        else:
            trials = 1
        if quack_log:
            h2_cmd += ' > h2.log'

        self.start_and_configure()
        time.sleep(SLEEP_S)

        if self.sidecar is not None:
            self.h2.cmdPrint(h2_cmd)
            for _ in range(trials - 1):
                self.start_quack_sender()
                time.sleep(SLEEP_S)
                self.h2.cmdPrint(h2_cmd)
        else:
            self.h2.cmdPrint(h2_cmd)

    def cli(self):
        CLI(self.net)

    def stop(self):
        if self.net is not None:
            self.net.stop()


if __name__ == '__main__':
    setLogLevel('info')

    parser = argparse.ArgumentParser(prog='Sidecar')
    parser.add_argument('--benchmark',
                        metavar='HTTP_VER',
                        help='Run a single benchmark rather than start the '
                             'CLI for the HTTP version [tcp|quic]')
    parser.add_argument('-p', '--pep', action='store_true',
                        help='Start a TCP pep on r1')
    parser.add_argument('--tso', action='store_true',
                        help='Enable TCP segment offloading (tso) and generic '
                             'segment offloading (gso). By default, both are '
                             'disabled')
    parser.add_argument('-cc',
                        default='cubic',
                        metavar='TCP_CC_ALG',
                        help='Sets the TCP and QUIC congestion control '
                             'mechanism [reno|cubic] (default: cubic)')
    parser.add_argument('--log-level',
                        default='error',
                        help='Sets the RUST_LOG level in the quACK sender '
                             '(if applicable) and the quiche client. '
                             '[error|warn|info|debug|trace] (default: error)')
    parser.add_argument('--delay1',
                        type=int,
                        default=75,
                        metavar='MS',
                        help='1/2 RTT between h1 and r1 (default: 75)')
    parser.add_argument('--delay2',
                        type=int,
                        default=1,
                        metavar='MS',
                        help='1/2 RTT between r1 and h2 (default: 1)')
    parser.add_argument('--loss1',
                        type=int,
                        default=0,
                        metavar='num',
                        help='loss (in %%) between h1 and r1 (default: 0)')
    parser.add_argument('--loss2',
                        type=int,
                        default=1,
                        metavar='num',
                        help='loss (in %%) between r1 and h2 (default: 1)')
    parser.add_argument('--bw1',
                        type=int,
                        default=10,
                        help='link bandwidth (in Mbps) between h1 and r1 '
                             '(default: 10)')
    parser.add_argument('--bw2',
                        type=int,
                        default=100,
                        help='link bandwidth (in Mbps) between r1 and h2 '
                             '(default: 100)')
    parser.add_argument('-s', '--sidecar',
                        help='If benchmark, enables the sidecar and sends '
                             'the quACK with the specified frequency. '
                             'Units are in terms of ms or packets e.g., '
                             '2ms or 2p')
    parser.add_argument('--threshold',
                        type=int,
                        default=20,
                        help='If benchmark, sets the quACK sender and '
                             'receiver to initialize their quACKs with '
                             'this threshold.')
    parser.add_argument('-n', '--nbytes',
                        default='1M',
                        metavar='num',
                        help='If benchmark, the number of bytes to run '
                        '(default: 1M)')
    parser.add_argument('-t', '--trials',
                        type=int,
                        metavar='num',
                        help='If benchmark, the number of trials')
    parser.add_argument('--stdout',
                        default='/dev/null',
                        metavar='FILENAME',
                        help='If benchmark, file to write curl stdout '
                             '(default: /dev/null)')
    parser.add_argument('--stderr',
                        default='/dev/null',
                        metavar='FILENAME',
                        help='If benchmark, file to write curl stderr '
                             '(default: /dev/null)')
    parser.add_argument('--iperf',
                        type=int,
                        metavar='TIME_S',
                        help='Run an iperf test for this length of time with '
                             'a server on h1 and client on h2.')
    parser.add_argument('--quack-log', action='store_true')
    args = parser.parse_args()
    sc = SidecarNetwork(args)
    sc.clean_logs()

    if args.iperf is not None:
        sc.iperf(args.iperf)
    elif args.benchmark is not None:
        sc.benchmark(args.nbytes, args.benchmark, args.trials, args.cc,
            args.stdout, args.stderr, args.quack_log)
    else:
        sc.start_and_configure()
        sc.cli()
    sc.stop()
