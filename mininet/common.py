import sys
import os


def mac(digit):
    assert 0 <= digit < 10
    return f'00:00:00:00:00:0{int(digit)}'

def ip(digit):
    assert 0 <= digit < 10
    return f'10.0.{int(digit)}.10/24'

def sclog(val):
    print(f'[sidecar] {val}', file=sys.stderr);

def popen(host, cmd):
    p = host.popen(cmd.split(' '))
    exitcode = p.wait()
    for line in p.stderr:
        sys.stderr.buffer.write(line)
    if exitcode != 0:
        print(f'{host}({cmd}) = {exitcode}')
        sys.stderr.buffer.write(b'\n')
        sys.stderr.buffer.flush()
        exit(1)

def get_max_queue_size_bytes(rtt_ms, bw_mbitps):
    bdp = rtt_ms * bw_mbitps * 1000000. / 1000. / 8.
    return bdp

def get_max_queue_size(rtt_ms, bw_mbitps):
    """
    Calculate the maximum queue size as
    Bandwidth Delay Product (BDP) / MTU * 1.1 packets.
    """
    bdp = get_max_queue_size_bytes(rtt_ms, bw_mbitps)
    mtu = 1500
    return int(bdp / mtu * 1.1) + 1

def clean_logs():
    os.system('rm -f r1.log h1.log h2.log f1.log f2.log')
    os.system('touch r1.log h1.log h2.log f1.log f2.log')

def estimate_timeout(n, proxy, quic, loss):
    """
    Timeout is linear in the data size, smaller if there is a proxy, larger if
    the client uses HTTP/3 instead of HTTP/1.1, larger when there is more loss,
    and has a floor of 15 seconds. Otherwise defaults to 5 minutes. Timeout is
    measured in seconds.
    """
    try:
        if 'k' in n:
            kb = int(n[:-1])
        elif 'M' in n:
            kb = int(n[:-1]) * 1000
        scale = 0.015
        if quic:
            scale *= 2
        if float(loss) > 1:
            scale *= float(loss) / 1.5
        if proxy:
            scale *= 0.1
        return max(int(scale * kb), 15)
    except:
        return 300
