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
    def __init__(self):
        self.net = Mininet(controller=None, link=TCLink)

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
        self.h1.cmd("tc qdisc add dev h1-eth0 root netem delay 300ms")
        self.h2.cmd("tc qdisc add dev h2-eth0 root netem loss 10% delay 1ms")
        self.r1.cmd("tc qdisc add dev r1-eth0 root netem delay 300ms")
        self.r1.cmd("tc qdisc add dev r1-eth1 root netem delay 1ms")

        # Start the webserver on h1
        self.h1.cmd("nginx -c ../webserver/nginx.conf")
        self.h1.cmd("python3 ../webserver/server.py &")

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
    sc = SidecarNetwork()
    sc.start()
    sc.cli()
    sc.stop()
