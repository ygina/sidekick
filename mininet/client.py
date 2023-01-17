import argparse
import os

def run_client(nbytes, http, trials, stdout, stderr, cc):
    cmd = './webserver/run_client.sh {} {} {} {} {} {}'.format(
        nbytes, http, trials, stdout, stderr, cc)
    os.system(cmd)

def check_trials(value):
    try:
        value = int(value)
        if value > 0:
            return value
    except:
        pass
    err = f'trials is not a positive integer: {value}'
    raise argparse.ArgumentTypeError(err)

def check_http(value):
    if value == 'tcp':
        return 1
    elif value == 'quic':
        return 3
    err = f'http version must be tcp or quic: {value}'
    raise argparse.ArgumentTypeError(err)

if __name__ == '__main__':
    parser = argparse.ArgumentParser(prog='Sidecar Client')
    parser.add_argument('-n',
                        required=True,
                        help='Number of bytes to send e.g. 1M')
    parser.add_argument('--http',
                        required=True,
                        help='HTTP version to use [tcp|quic]',
                        type=check_http)
    parser.add_argument('-t', '--trials',
                        help='Number of trials',
                        type=check_trials)
    parser.add_argument('--stdout',
                        default='/dev/null',
                        metavar='FILE',
                        help='File to write stdout to (default: /dev/null)')
    parser.add_argument('--stderr',
                        default='/dev/null',
                        metavar='FILE',
                        help='File to write stderr to (default: /dev/null)')
    parser.add_argument('-cc',
                        default='cubic',
                        metavar='TCP_CC_ALG',
                        help='Sets the TCP and QUIC congestion control '
                             'mechanism [reno|cubic] (default: cubic)')
    args = parser.parse_args()

    run_client(nbytes=args.n, http=args.http, trials=args.trials,
        stdout=args.stdout, stderr=args.stderr, cc=args.cc)
