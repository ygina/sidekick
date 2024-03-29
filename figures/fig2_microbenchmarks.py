import os
import re
from collections import defaultdict
from common import *

def plot_graph(xs, data, keys, outdir, colors, linestyles, xlabel, ylabel, legend, pdf=None):
    plt.figure(figsize=(6, 4))
    for (i, key) in enumerate(keys):
        if len(xs) != len(data[key]):
            import pdb; pdb.set_trace()
        plt.plot(xs, data[key], label=key, color=colors[i], linestyle=linestyles[i])
    plt.xlabel(xlabel)
    plt.ylabel(ylabel)
    plt.xlim(0)
    plt.ylim(0)
    plt.grid()
    if legend:
        plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.3), ncol=3,
                   labelspacing=0, handletextpad=0.1, columnspacing=0.5)
    # plt.title(pdf)
    if pdf:
        save_pdf(f'{outdir}/{pdf}')
    plt.clf()

def parse_decode_output(filename, x_regex, expected_xs):
    x_to_y = defaultdict(lambda: None)
    x = None
    os.system(f'touch {filename}')
    with open(filename) as f:
        lines = f.read().split('\n')
    for line in lines:
        match = re.match(x_regex, line)
        if match is not None:
            x = int(match.group(1))
            continue
        match = re.match(r'.*avg = (\S+)', line)
        if match is not None:
            y = match.group(1)  # convert to μs
            if 'ms' in y:
                x_to_y[x] = float(y[:-2]) * 1000
            elif 'µs' in y:
                x_to_y[x] = float(y[:-2])
            else:
                print(y)
                raise Exception
    return [x_to_y[x] for x in expected_xs]

def parse_construct_output(filename, x_regex, expected_xs):
    x_to_y = defaultdict(lambda: None)
    x = None
    os.system(f'touch {filename}')
    with open(filename) as f:
        lines = f.read().split('\n')
    for line in lines:
        match = re.match(x_regex, line)
        if match is not None:
            x = int(match.group(1))
            continue
        match = re.match(r'.*\(per-packet\): (\S+/packet) .*', line)
        if match is not None:
            y = match.group(1)[:-7]  # convert to ns/pkt
            if 'ns' in y:
                x_to_y[x] = float(y[:-2])
            elif 'µs' in y:
                x_to_y[x] = float(y[:-2]) * 1000.
            else:
                print(y)
                raise Exception
    return [x_to_y[x] for x in expected_xs]

def plot_num_candidates_vs_decode_time_method(args, pdf):
    results = f'{args.logdir}/quack/num_candidates_vs_decode_time'
    os.system(f'mkdir -p {results}')

    xs = [x for x in range(1000, 50001, 1000)]
    keys = ['PlugInRoots', 'PolyFactor']

    # m = 20, n = 0 to 40k, t = 20, b = 32 bits
    data = {}
    x_regex = r'.*-n (\d+)'
    for key in keys:
        filename = f'{results}/{key}.txt'
        print(filename)
        ys = parse_decode_output(filename, x_regex, xs)
        if None not in ys:
            data[key] = ys
            continue
        if not args.execute:
            print(f'{filename} {len(xs) - ys.count(None)}/{len(xs)} points')
            return
        for (i, x) in enumerate(xs):
            if ys[i] is None:
                cmd = ['./target/release/examples/benchmark_decode', 'power-sum']
                cmd += ['-d', '20', '-t', '20', '-b', '32', '--trials', str(args.trials)]
                cmd += ['-n', str(x)]
                if key == 'PolyFactor':
                    cmd += ['--factor']
                execute_experiment(cmd, filename, cwd=args.workdir)
        data[key] = parse_decode_output(filename, x_regex, xs)
        assert len(data[key]) == len(xs)

    # data['PlugInRoots'] = [0.273832,0.550482,0.819121,1.091126,1.354429,1.626995,1.902114,2.166587,2.441659,2.695782,2.967711,3.252944,3.510062,3.794189,4.050174,4.334082,4.598469,4.866202,5.144076,5.400785,5.661248,5.938144,6.212073,6.477512,6.743419,7.003082,7.281669,7.555356,7.810546,8.096134,8.350279,8.648749,8.939695,9.178242,9.451259,9.71894,9.977424,10.292975,10.515204,10.789034]
    # data['PolyFactor'] = [4.011627,4.085793,4.238399,4.320376,4.41314,4.54784,4.656416,4.762687,4.87368,4.997019,5.127364,5.222205,5.348893,5.438783,5.546157,5.649644,5.782371,5.853962,5.950526,6.123299,6.230779,6.373078,6.428554,6.627189,6.739125,6.895649,6.934072,7.082421,7.170848,7.233888,7.38107,7.500699,7.612108,7.758209,7.914039,8.095923,8.042781,8.132017,8.322325,8.403114]
    plot_graph(xs, data, keys,
               outdir=args.outdir,
               colors=[COLOR_MAP['quack'], colors[5]],
               linestyles=[LINESTYLES[1], LINESTYLES[0]],
               xlabel='Num Sent Packets',
               ylabel='Decode Time (μs)',
               legend=args.legend, pdf=pdf)

def plot_num_candidates_vs_decode_time(args, pdf):
    results = f'{args.logdir}/quack/num_candidates_vs_decode_time'
    os.system(f'mkdir -p {results}')

    xs = [x for x in range(10, 301, 5)]
    num_bits = [16, 32, 64]

    data = {}
    x_regex = r'.*-n (\d+)'
    for key in num_bits:
        key_str = f'b={int(key/8)}'
        filename = f'{results}/{key}.txt'
        print(filename)
        ys = parse_decode_output(filename, x_regex, xs)
        if None not in ys:
            data[key_str] = ys
            continue
        if not args.execute:
            print(f'{filename} {len(xs) - ys.count(None)}/{len(xs)} points')
            return
        for (i, x) in enumerate(xs):
            if ys[i] is None:
                cmd = ['./target/release/examples/benchmark_decode', 'power-sum']
                cmd += ['-d', '10', '-t', '10', '--trials', str(args.trials)]
                cmd += ['-b', str(key), '-n', str(x)]
                if key == 16:
                    cmd += ['--precompute']
                elif key == 64:
                    cmd += ['--montgomery']
                execute_experiment(cmd, filename, cwd=args.workdir)
        data[key_str] = parse_decode_output(filename, x_regex, xs)
        assert len(data[key_str]) == len(xs)

    plot_graph(xs, data, ['b=2', 'b=4', 'b=8'],
               outdir=args.outdir,
               colors=[colors[6], COLOR_MAP['quack'], colors[7]],
               linestyles=[LINESTYLES[2], LINESTYLES[1], LINESTYLES[3]],
               xlabel='Num Sent Packets',
               ylabel='Decode Time (μs)',
               legend=args.legend, pdf=pdf)

def plot_num_missing_vs_decode_time(args, pdf):
    results = f'{args.logdir}/quack/num_missing_vs_decode_time'
    os.system(f'mkdir -p {results}')

    xs = [x for x in range(5, 301, 5)]
    num_bits = [16, 32, 64]

    data = {}
    x_regex = r'.*-d (\d+).*'
    for key in num_bits:
        key_str = f'b={int(key/8)}'
        filename = f'{results}/{key}.txt'
        print(filename)
        ys = parse_decode_output(filename, x_regex, xs)
        if None not in ys:
            data[key_str] = ys
            continue
        if not args.execute:
            print(f'{filename} {len(xs) - ys.count(None)}/{len(xs)} points')
            continue
        for (i, x) in enumerate(xs):
            if ys[i] is None:
                cmd = ['./target/release/examples/benchmark_decode', 'power-sum']
                cmd += ['-n', '300', '--trials', str(args.trials)]
                cmd += ['-d', str(x), '-t', str(x)]
                cmd += ['-b', str(key)]
                if key == 16:
                    cmd += ['--precompute']
                elif key == 64:
                    cmd += ['--montgomery']
                execute_experiment(cmd, filename, cwd=args.workdir)
        data[key_str] = parse_decode_output(filename, x_regex, xs)
        assert len(data[key_str]) == len(xs)

    # data['b=2'] = [0.051497,0.076008,0.100874,0.1193,0.13093,0.151181,0.178237,0.200465,0.218228,0.232929,0.250242,0.274231,0.286133,0.303111,0.316404,0.341572,0.368286,0.377316,0.400136,0.419409,0.434085,0.468943,0.483073,0.500438,0.516458,0.528489,0.567682,0.589233,0.606814,0.610339]
    # data['b=4'] = [0.122179,0.274451,0.439462,0.590673,0.744664,0.914965,1.065969,1.218622,1.372891,1.563994,1.696404,1.866024,2.019276,2.180499,2.339186,2.527871,2.663722,2.823141,3.008938,3.176965,3.330336,3.499189,3.66254,3.853589,4.004775,4.165358,4.333277,4.512102,4.683568,4.857881]
    # data['b=8'] = [0.110077,0.238967,0.36906,0.510326,0.647589,0.772023,0.921055,1.036879,1.172135,1.31098,1.44418,1.579808,1.717533,1.852751,1.998262,2.121355,2.257078,2.401623,2.519603,2.654495,2.812686,2.951031,3.082817,3.236986,3.353193,3.498203,3.637827,3.779945,3.924769,4.067958]
    plot_graph(xs, data, ['b=2', 'b=4', 'b=8'],
               outdir=args.outdir,
               colors=[colors[6], COLOR_MAP['quack'], colors[7]],
               linestyles=[LINESTYLES[2], LINESTYLES[1], LINESTYLES[3]],
               xlabel='Num Missing Packets',
               ylabel='Decode Time (μs)',
               legend=args.legend, pdf=pdf)

def plot_threshold_vs_encode_time(args, pdf):
    results = f'{args.logdir}/quack/threshold_vs_encode_time'
    os.system(f'mkdir -p {results}')

    xs = [x for x in range(10, 310, 10)]
    num_bits = [16, 32, 64]

    data = {}
    x_regex = r'.*-t (\d+).*'
    for key in num_bits:
        key_str = f'b={int(key/8)}'
        filename = f'{results}/{key}.txt'
        print(filename)
        ys = parse_construct_output(filename, x_regex, xs)
        if None not in ys:
            data[key_str] = ys
            continue
        if not args.execute:
            print(f'{filename} {len(xs) - ys.count(None)}/{len(xs)} points')
            continue
        for (i, x) in enumerate(xs):
            if ys[i] is None:
                cmd = ['./target/release/examples/benchmark_construct', 'power-sum']
                cmd += ['-e', '1000', '--trials', str(args.trials)]
                cmd += ['-t', str(x)]
                cmd += ['-b', str(key)]
                if key == 16:
                    cmd += ['--precompute']
                elif key == 64:
                    cmd += ['--montgomery']
                execute_experiment(cmd, filename, cwd=args.workdir)
        data[key_str] = parse_construct_output(filename, x_regex, xs)
        assert len(data[key_str]) == len(xs)

    # data['b=2'] = [0.003,0.127,0.133,0.148,0.162,0.162,0.177,0.197,0.188,0.178,0.215,0.215,0.270,0.262,0.264,0.282,0.299,0.302,0.334,0.339,0.370,0.363,0.386,0.392,0.396,0.400,0.433,0.455,0.457,0.455,0.456]
    # data['b=4'] = [0.0,0.054,0.112,0.180,0.238,0.297,0.364,0.424,0.490,0.551,0.614,0.668,0.731,0.805,0.859,0.924,0.981,1.04,1.11,1.163,1.235,1.293,1.351,1.423,1.482,1.538,1.602,1.656,1.726,1.788,1.851]
    # data['b=8'] = [0,0.050,0.109,0.163,0.224,0.294,0.345,0.406,0.460,0.514,0.584,0.633,0.698,0.750,0.811,0.867,0.932,0.981,1.042,1.102,1.156,1.225,1.274,1.35,1.396,1.45,1.513,1.563,1.618,1.678,1.742]
    for key in data:
        data[key] = [x for x in data[key]]
    plot_graph(xs, data, [x for x in sorted(data.keys())],
               outdir=args.outdir,
               colors=[colors[6], COLOR_MAP['quack'], colors[7]],
               linestyles=[LINESTYLES[2], LINESTYLES[1], LINESTYLES[3]],
               xlabel='Threshold (pkts)',
               ylabel='Encode Time (ns/pkt)',
               legend=args.legend, pdf=pdf)

if __name__ == '__main__':
    parser.add_argument('-t', '--trials', default=100, type=int,
                        help='Number of trials (default: 100)')
    args = parser.parse_args()

    plot_num_candidates_vs_decode_time(args, pdf='fig2a_quack_num_candidates_vs_decode_time.pdf')
    plot_num_missing_vs_decode_time(args, pdf='fig2b_quack_num_missing_vs_decode_time.pdf')
    plot_threshold_vs_encode_time(args, pdf='fig2c_quack_threshold_vs_encode_time.pdf')
