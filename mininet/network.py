import sys
import time
import re
import os
import subprocess
from common import *
from mininet.net import Mininet
from mininet.link import TCLink


class SidecarNetwork():
    def __init__(self, delay1, delay2, loss1, loss2, bw1, bw2, qdisc):
        self.net = Mininet(controller=None, link=TCLink)

        # Add hosts and switches
        self.h1 = self.net.addHost('h1', ip=ip(1), mac=mac(1))
        self.h2 = self.net.addHost('h2', ip=ip(2), mac=mac(2))
        self.r1 = self.net.addHost('r1')

        # Add links
        self.net.addLink(self.r1, self.h1)
        self.net.addLink(self.r1, self.h2)
        self.net.build()

        # Configure interfaces
        popen(self.r1, "ifconfig r1-eth0 0")
        popen(self.r1, "ifconfig r1-eth1 0")
        popen(self.r1, "ifconfig r1-eth0 hw ether 00:00:00:00:01:01")
        popen(self.r1, "ifconfig r1-eth1 hw ether 00:00:00:00:01:02")
        popen(self.r1, "ip addr add 10.0.1.1/24 brd + dev r1-eth0")
        popen(self.r1, "ip addr add 10.0.2.1/24 brd + dev r1-eth1")
        self.r1.cmd("echo 1 > /proc/sys/net/ipv4/ip_forward")
        popen(self.h1, "ip route add default via 10.0.1.1")
        popen(self.h2, "ip route add default via 10.0.2.1")

        # Configure link latency, delay, bandwidth, and queue size
        # https://unix.stackexchange.com/questions/100785/bucket-size-in-tbf
        rtt_ms = 2 * (delay1 + delay2)
        bw_mbps = min(bw1, bw2)
        bdp = get_max_queue_size_bytes(rtt_ms, bw_mbps)
        print(f'max_queue_size (bytes) = {bdp}')
        def tc(host, iface, loss, delay, bw):
            if qdisc == 'tbf':
                popen(host, f'tc qdisc add dev {iface} root handle 1:0 ' \
                            f'netem loss {loss}% delay {delay}ms')
                popen(host, f'tc qdisc add dev {iface} parent 1:1 handle 10: ' \
                            f'tbf rate {bw}mbit burst {bw*500*2} limit {bdp}')
            elif qdisc == 'cake':
                popen(host, f'tc qdisc add dev {iface} root handle 1:0 ' \
                            f'netem loss {loss}% delay {delay}ms')
                popen(host, f'tc qdisc add dev {iface} parent 1:1 handle 10: ' \
                            f'cake bandwidth {bw}mbit' \
                            f'oceanic flowblind besteffort')
            elif qdisc == 'codel':
                popen(host, f'tc qdisc add dev {iface} root handle 1:0 ' \
                            f'netem loss {loss}% delay {delay}ms rate {bw}mbit')
                popen(host, f'tc qdisc add dev {iface} parent 1:1 handle 10: codel')
            elif qdisc == 'red':
                popen(host, f'tc qdisc add dev {iface} handle 1:0 root ' \
                            f'red limit {bdp*4} avpkt 1000 adaptive ' \
                            f'harddrop bandwidth {bw}Mbit')
                popen(host, f'tc qdisc add dev {iface} parent 1:1 handle 10: ' \
                            f'netem loss {loss}% delay {delay}ms rate {bw}mbit')
            elif qdisc == 'grenville':
                popen(host, f'tc qdisc add dev {iface} root handle 2: netem loss {loss}% delay {delay}ms')
                popen(host, f'tc qdisc add dev {iface} parent 2: handle 3: htb default 10')
                popen(host, f'tc class add dev {iface} parent 3: classid 10 htb rate {bw}Mbit')
                popen(host, f'tc qdisc add dev {iface} parent 3:10 handle 11: ' \
                            f'red limit {bdp*4} avpkt 1000 adaptive harddrop bandwidth {bw}Mbit')
            else:
                sclog('{} {} no qdisc enabled'.format(host, iface))

        tc(self.h1, 'h1-eth0', loss1, delay1, bw1)
        tc(self.r1, 'r1-eth0', loss1, delay1, bw1)
        tc(self.r1, 'r1-eth1', loss2, delay2, bw2)
        tc(self.h2, 'h2-eth0', loss2, delay2, bw2)

        # Start the webserver on h1
        sclog('Starting the NGINX/Python webserver on h1...')
        self.h1.cmd("kill $(pidof nginx)")
        home_dir = os.environ['HOME']
        popen(self.h1, f'nginx -c {home_dir}/sidecar/webserver/nginx.conf')
        self.h1.cmd("python3 webserver/server.py >> h1.log 2>&1 &")
        while True:
            with open('h1.log', 'r') as f:
                if 'Starting httpd' in f.read():
                    return
            time.sleep(0.1)

    def set_segmentation_offloading(self, on):
        """
        Turn off tso and gso to send MTU-sized packets
        """
        sclog('tso and gso are {}'.format('ON' if on else 'OFF'))
        x = 'on' if on else 'off'
        popen(self.h1, f'ethtool -K h1-eth0 gso {x} tso {x}')
        popen(self.h2, f'ethtool -K h2-eth0 gso {x} tso {x}')
        popen(self.r1, f'ethtool -K r1-eth0 gso {x} tso {x}')
        popen(self.r1, f'ethtool -K r1-eth1 gso {x} tso {x}')

    def start_tcp_pep(self):
        sclog('Starting the TCP PEP on r1...')
        popen(self.r1, 'ip rule add fwmark 1 lookup 100')
        popen(self.r1, 'ip route add local 0.0.0.0/0 dev lo table 100')
        popen(self.r1, 'iptables -t mangle -F')
        popen(self.r1, 'iptables -t mangle -A PREROUTING -i r1-eth1 -p tcp -j TPROXY --on-port 5000 --tproxy-mark 1')
        popen(self.r1, 'iptables -t mangle -A PREROUTING -i r1-eth0 -p tcp -j TPROXY --on-port 5000 --tproxy-mark 1')
        self.r1.cmd('pepsal -v >> r1.log 2>&1 &')

    def start_quack_sender(self, frequency, threshold):
        """
        - `frequency`: frequency of the sidecar sender e.g. 2ms or 2p
        - `threshold`: quACK threshold
        """
        print('', file=sys.stderr)
        sclog('Starting the QUIC sidecar sender on r1...')
        if 'ms' in frequency:
            frequency = re.match(r'(\d+)ms', frequency).group(1)
            frequency = f'--frequency-ms {frequency}'
        elif 'p' in frequency:
            frequency = re.match(r'(\d+)p.*', frequency).group(1)
            frequency = f'--frequency-pkts {frequency}'
        else:
            raise 'Invalid frequency: {}'.format(frequency)

        self.r1.cmd(f'kill $(pidof sidecar)')
        # Does ./target/release/sender exist?
        print(self.r1.cmd(f'RUST_BACKTRACE=1 RUST_LOG=debug ' \
            f'./target/release/sender -i r1-eth1 -t {threshold} ' + \
            f'--target-addr 10.0.2.10:5103 ' + \
            f'{frequency} >> r1.log 2>&1 &'))

    def stop(self):
        if self.net is not None:
            self.net.stop()


def run_ping(net, num_pings):
    """
    Run a ping reachability test between all hosts.
    """
    net.h2.cmdPrint(f'ping -c{num_pings} 10.0.1.10')
    # net.h1.cmdPrint(f'ping -c{num_pings} 10.0.2.10')
    # net.r1.cmdPrint(f'ping -c{num_pings} 10.0.1.10')
    # net.h1.cmdPrint(f'ping -c{num_pings} 10.0.1.1')
    # net.r1.cmdPrint(f'ping -c{num_pings} 10.0.2.10')
    # net.h2.cmdPrint(f'ping -c{num_pings} 10.0.2.1')


def run_ss(net, time_s, host, interval_s=0.1):
    """
    Run an ss test to collect statistics about the TCP cwnd over time. Start
    the TCP client in the background and collect statistics every 0.1 seconds.
    There may have been a PEP started.
    """
    if host == 'r1':
        host = net.r1
    elif host == 'h2':
        host = net.h2
    else:
        exit(1)

    cmd = f'python3 mininet/client.py -n 50M tcp -t 1 --timeout {time_s+1} &'
    net.h2.cmd(cmd)
    for _ in range(int(time_s / interval_s)):
        host.cmdPrint('ss -t -i | grep -A1 "10.0.1.10:https$" | grep cwnd')
        time.sleep(interval_s)


def run_iperf(net, time_s, host):
    net.h1.cmd('iperf3 -s -f m > /dev/null 2>&1 &')
    if host == 'r1':
        host = net.r1
    elif host == 'h2':
        host = net.h2
    else:
        exit(1)
    host.cmdPrint(f'iperf3 -c 10.0.1.10 -t {time_s} -f m -b 20M -C cubic -i 0.1')


def run_multiflow(net, f1, f2, delay, threshold=20):
    """
    o = currently possible
    x = needs to be implemented
    - = impossible

          pep quack quic tcp
    pep   o   o     o    -
    quack -   x     o    o
    quic  -   -     o    o
    tcp   -   -     -    o
    """
    assert args.nbytes is not None
    assert not (f1 == 'quack' and f2 == 'quack')
    assert not (f1 == 'tcp' and f2 == 'pep')
    assert not (f1 == 'pep' and f2 == 'tcp')
    if 'pep' in [f1, f2]:
        net.start_tcp_pep()
    if 'quack' in [f1, f2]:
        net.start_quack_sender('2ms', threshold=threshold)

    def make_cmd(bm):
        if bm in ['tcp', 'pep']:
            client = 'tcp'
        elif bm in ['quic', 'quack']:
            client = 'quic'
        else:
            raise f'invalid benchmark: {bm}'
        cmd = ['python3', 'mininet/client.py', '-n', args.nbytes,
               '--stdout', args.stdout, '--stderr', args.stderr,
               '-t', '1', client]
        return cmd

    f1_cmd = make_cmd(f1)
    f2_cmd = make_cmd(f2)

    home_dir = os.environ['HOME']
    prefix = f'{home_dir}/sidecar/results/multiflow/loss{net.loss2}p'
    pcap_file = f'{prefix}/{f1}_{f2}_{args.nbytes}_delay{args.delay}s_bw{args.bw2}.pcap'
    os.system(f'mkdir -p {prefix}')
    os.system(f'rm -f {pcap_file}')
    net.h1.cmd(f"tcpdump -w {pcap_file} -i h1-eth0 'ip src 10.0.2.10 and (tcp or udp)' &")
    p1 = net.h2.popen(f1_cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    time.sleep(args.delay)
    p2 = net.h2.popen(f2_cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)

    def wait(p, logfile, i, bm):
        with open(logfile, 'ab') as f:
            for line in p.stdout:
                f.write(line)
                if b'200' in line:
                    sys.stdout.buffer.write(line.strip())
                    sys.stdout.buffer.write(
                        bytes(f'\t\t(flow{i}={bm})\n', 'utf-8'))
                    sys.stdout.buffer.flush()
                if b'time_total' in line and b'sidecurl' not in line:
                    sys.stdout.buffer.write(line)
                    sys.stdout.buffer.flush()
        p.wait()

    wait(p1, 'f1.log', 1, f1)
    wait(p2, 'f2.log', 2, f2)
    print(pcap_file)

