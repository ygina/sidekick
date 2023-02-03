import argparse
import os
import tempfile

def print_and_run_cmd(cmd):
    print(cmd)
    return os.system(cmd)

def estimate_timeout(nbytes, http, loss):
    try:
        if 'k' in nbytes:
            nbytes = int(nbytes[:-1])
        scale = 0.005
        if '1.1' in http:
            scale *= 2
        if loss == '2' or loss == '5':
            scale *= int(loss)
        return max(int(scale * nbytes), 15)
    except:
        return 3000

def run_client(nbytes, http, trials, stdout, stderr, cc, addr, sidecar, log_level, loss=None):
    f = tempfile.NamedTemporaryFile()
    print_and_run_cmd(f'head -c {nbytes} /dev/urandom > {f.name}')
    print(f'Data Size: {nbytes}')
    print(f'HTTP: {http}')
    # curl = 'curl-exp'
    curl = 'sidecurl'
    if args.sidecar is None:
        sidecar = ''
    else:
        sidecar = f'--sidecar {args.sidecar[0]} --threshold {args.sidecar[1]}'
    if trials is None:
        fmt="\n\n      time_connect:  %{time_connect}s\n   time_appconnect:  %{time_appconnect}s\ntime_starttransfer:  %{time_starttransfer}s\n                   ----------\n        time_total:  %{time_total}s\n\nexitcode: %{exitcode}\nresponse_code: %{response_code}\nsize_upload: %{size_upload}\nsize_download: %{size_download}\nerrormsg: %{errormsg}\n"
        cmd=f'RUST_LOG={log_level} {curl} {http} --insecure {cc} --data-binary @{f.name} {sidecar} https://{addr}/ -w \"{fmt}\"'
        print_and_run_cmd(f'eval \'{cmd}\'')
    else:
        fmt="%{time_connect}\\t%{time_appconnect}\\t%{time_starttransfer}\\t\\t%{time_total}\\t%{exitcode}\\t\\t%{response_code}\\t\\t%{size_upload}\\t\\t%{size_download}\\t%{errormsg}\\n"
        timeout = estimate_timeout(nbytes, http, loss)
        # cmd=f'RUST_LOG={log_level} QLOGDIR=/home/gina/sidecar/qlog '+\
        cmd=f'RUST_LOG={log_level} '+\
            f'{curl} {http} --insecure {cc} '+\
            f'--data-binary @{f.name} {sidecar} https://{addr}/ '+\
            f'-w \"{fmt}\" --max-time {timeout} -o {stdout} 2>>{stderr}'
        # cmd = f"/usr/bin/time -f\"0\t\t0\t\t0\t\t\t%e\t0\t200\" "+\
        #       f"/home/gina/quiche-sidecar/target/release/quiche-client "+\
        #       f"--max-data 100000000 "+\
        #       f"--no-verify https://10.0.1.10:443 --body {f.name} > /dev/null"
        header = 'time_connect\ttime_appconnect\ttime_starttransfer\ttime_total\texitcode\tresponse_code\tsize_upload\tsize_download\terrormsg'
        print(cmd)
        print(header)
        for _ in range(trials):
            os.system(f'eval \'{cmd}\'')

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
    try:
        value = int(value)
        if value == 1:
            return '--http1.1'
        elif value == 2:
            return '--http2'
        elif value == 3:
            return '--http3'
    except:
        pass
    err = f'http version must be 1, 2, or 3: {value}'
    raise argparse.ArgumentTypeError(err)

def check_cc(value):
    if value == '':
        return ''
    if value not in ['reno', 'cubic']:
        err = f'tcp congestion control algorithm must be reno or cubic: {value}'
        raise argparse.ArgumentTypeError(err)
    return f'--quiche-cc {value}'

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
    parser.add_argument('-s', '--sidecar',
                        metavar=('IFACE', 'THRESHOLD'),
                        nargs=2,
                        help='Sidecar interface that packets are being sent on '
                             'and the quACK threshold.')
    parser.add_argument('--log-level',
                        default='error',
                        help='Sets the RUST_LOG level in the quiche client. '
                             '[error|warn|info|debug|trace] (default: error)')
    parser.add_argument('--stdout',
                        default='/dev/null',
                        metavar='FILE',
                        help='File to write stdout to (default: /dev/null)')
    parser.add_argument('--stderr',
                        default='/dev/null',
                        metavar='FILE',
                        help='File to write stderr to (default: /dev/null)')
    parser.add_argument('-cc',
                        default='',
                        metavar='TCP_CC_ALG',
                        type=check_cc,
                        help='Sets the TCP and QUIC congestion control '
                             'mechanism [reno|cubic] (default: cubic)')
    parser.add_argument('--addr',
                        default='10.0.1.10:443',
                        help='Server address (default: 10.0.1.10:443)')
    parser.add_argument('--loss',
                        help='Loss percentage, used to estimate timeout')
    args = parser.parse_args()

    run_client(nbytes=args.n, http=args.http, trials=args.trials,
        stdout=args.stdout, stderr=args.stderr, cc=args.cc, addr=args.addr,
        sidecar=args.sidecar, log_level=args.log_level, loss=args.loss)
