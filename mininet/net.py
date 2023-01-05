import argparse
import logging
from mininet.net import Mininet
from mininet.cli import CLI
from mininet.log import setLogLevel
from mininet.link import TCLink

def mac(digit):
    assert 0 <= digit < 10
    return '00:00:00:00:00:0{}'.format(int(digit))

def ip(digit):
    assert 0 <= digit < 10
    return '10.0.{}.10/24'.format(int(digit))

class SidecarNetwork():
    def __init__(self, args):
        self.net = Mininet(controller=None, link=TCLink)
        self.pep = args.pep
        self.delay1 = int(args.delay1)
        self.delay2 = int(args.delay2)
        self.loss2 = int(args.loss2)

        # Add hosts and switches
        self.h1 = self.net.addHost('h1', ip=ip(1), mac=mac(1))
        self.h2 = self.net.addHost('h2', ip=ip(2), mac=mac(2))
        self.r1 = self.net.addHost('r1')

        # Add links
        self.net.addLink(self.r1, self.h1)
        self.net.addLink(self.r1, self.h2)
        self.net.build()

    def start(self):
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
        self.h1.cmd('tc qdisc add dev h1-eth0 root netem delay {}ms'.format(self.delay1))
        self.h2.cmd('tc qdisc add dev h2-eth0 root netem loss {}% delay {}ms'.format(self.loss2, self.delay2))
        self.r1.cmd('tc qdisc add dev r1-eth0 root netem delay {}ms'.format(self.delay1))
        self.r1.cmd('tc qdisc add dev r1-eth1 root netem delay {}ms'.format(self.delay2))

        # Start the webserver on h1
        # TODO: not user-dependent path
        print('[sidecar] Starting the NGINX/Python webserver on h1...')
        self.h1.cmd("nginx -c /home/gina/sidecar/webserver/nginx.conf")
        self.h1.cmd("python3 webserver/server.py &")

        # Start the TCP PEP on r1
        if self.pep:
            print('[sidecar] Starting the TCP PEP on r1...')
            self.r1.cmd('ip rule add fwmark 1 lookup 100')
            self.r1.cmd('ip route add local 0.0.0.0/0 dev lo table 100')
            self.r1.cmd('iptables -t mangle -F')
            self.r1.cmd('iptables -t mangle -A PREROUTING -i r1-eth1 -p tcp -j TPROXY --on-port 5000 --tproxy-mark 1')
            self.r1.cmd('iptables -t mangle -A PREROUTING -i r1-eth0 -p tcp -j TPROXY --on-port 5000 --tproxy-mark 1')
            self.r1.cmd('pepsal -v &')
        else:
            logging.info('NOT starting the TCP PEP')

        #self.h1.cmd("tc qdisc add dev h1-eth0 root netem delay 250ms 25ms distribution normal")
        # self.h2.cmd("tc qdisc add dev h2-eth0 root netem delay 30ms 3ms distribution normal")
        #self.h2.cmd("tc qdisc add dev h2-eth0 root netem loss 10% delay 30ms 3ms distribution normal")
        #self.r1.cmd("tc qdisc add dev r1-eth0 root netem delay 250ms 25ms distribution normal")
        #self.r1.cmd("tc qdisc add dev r1-eth1 root netem delay 30ms 3ms distribution normal")

    def cli(self):
        CLI(self.net)

    def stop(self):
        self.net.stop()


if __name__ == '__main__':
    setLogLevel('info')

    parser = argparse.ArgumentParser(prog='Sidecar')
    parser.add_argument('-p', '--pep', action='store_true')
    parser.add_argument('-d1', '--delay1',
                        default=300,
                        help='1/2 RTT (in ms) between h1 and r1')
    parser.add_argument('-d2', '--delay2',
                        default=1,
                        help='1/2 RTT (in ms) between r1 and h2')
    parser.add_argument('-l2', '--loss2',
                        default=10,
                        help='loss (in %%) between r1 and h2')
    args = parser.parse_args()

    sc = SidecarNetwork(args)
    sc.start()
    sc.cli()
    sc.stop()
