import argparse
import sys
import time
from network import *
from mininet.log import setLogLevel

if __name__ == '__main__':
    setLogLevel('info')

    parser = argparse.ArgumentParser(prog='Benchmark Encoder')

    ############################################################################
    # Network Configurations
    parser.add_argument('--tput', default=500, type=int, metavar='PPS',
        help='Target load generator throughput in packets per second. The load '
             'generator may not be able to achieve too high of throughputs. '
             '(default: 500)')
    parser.add_argument('--length', '-l', default=70, type=int, metavar='BYTES',
        help='Target load generator packet length, the -l option in iperf3 '
             '(default: 70)')
    parser.add_argument('--warmup', '-w', type=int, default=3, metavar='S',
        help='Warmup time, in seconds (default: 3)')
    parser.add_argument('--timeout', '-t', type=int, default=5, metavar='S',
        help='Timeout, in seconds (default: 5)')
    parser.add_argument('--frequency', type=int, default=0,
        help='Quack frequency in ms (default: 0)')
    parser.add_argument('--threshold', type=int, default=20, metavar='PACKETS',
        help='Quack threshold (default: 20)')
    parser.add_argument('--disable-sidecar', action='store_true',
        help='Disable the sidecar to test only iperf load generator')

    args = parser.parse_args()
    net = SidecarNetwork(delay1=0, delay2=0, loss1=0, loss2=0, bw1=0,
        bw2=0, qdisc='none')

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
    h1 = net.h1.popen(f'taskset -c 1 iperf3 -s -f m'.split(' '),
        stderr=subprocess.STDOUT, stdout=subprocess.PIPE)
    # load_generator = net.h2.popen(f'./target/release/load_generator --warmup {args.warmup} --tput {args.tput}'.split(' '))
    client_cmd = f'taskset -c 2 iperf3 -c 10.0.1.10 --udp --time {args.warmup + args.timeout + 1} --congestion cubic'.split(' ')
    client_cmd += ['-b', str(int(args.tput * args.length * 8))]
    client_cmd += ['-l', str(args.length)]
    sclog(f'Target rate is {args.tput * args.length} bytes/s')
    h2 = net.h2.popen(client_cmd,
        stderr=subprocess.STDOUT, stdout=subprocess.PIPE)
    time.sleep(args.warmup)
    if not args.disable_sidecar:
        env = os.environ.copy()
        # env['RUST_LOG'] = 'trace'
        r1 = net.r1.popen(f'taskset -c 3 ./target/release/benchmark_encode --threshold {args.threshold} --frequency {args.frequency}'.split(' '),
            stderr=subprocess.STDOUT, stdout=subprocess.PIPE, env=env)
    time.sleep(args.timeout)
    if not args.disable_sidecar:
        r1.terminate()
    sys.stdout.buffer.write(h1.stdout.peek())
    sys.stdout.buffer.write(b'\n')
    sys.stdout.buffer.write(h2.stdout.peek())
    sys.stdout.buffer.write(b'\n')
    sys.stdout.buffer.flush()

    if not args.disable_sidecar:
        for line in r1.stdout.peek().split(b'\n'):
            if b'DONE' in line:
                break
            sys.stdout.buffer.write(line)
            sys.stdout.buffer.write(b'\n')
            sys.stdout.buffer.flush()
    net.stop()
