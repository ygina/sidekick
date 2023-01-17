import os.path
import statistics
from os import path
from common import *

# DATE = '010922'
DATE = ''
NUM_TRIALS = 20
TARGET_XS = [x for x in range(200, 1000, 200)] + [x for x in range(2000, 40001, 2000)]

class DataPoint:
    def __init__(self, arr, normalize=None):
        if normalize is not None:
            arr = [normalize * 1. / x for x in arr]
        arr.sort()
        mid = int(len(arr) / 2)
        self.p50 = statistics.median(arr)
        if len(arr) % 2 == 1:
            self.p25 = statistics.median(arr[:mid+1])
        else:
            self.p25 = statistics.median(arr[:mid])
        self.p75 = statistics.median(arr[mid:])
        self.minval = arr[0]
        self.maxval = arr[-1]
        self.avg = statistics.mean(arr)
        self.stdev = None if len(arr) == 1 else statistics.stdev(arr)

def parse_data(filename, normalize, data_key='time_total'):
    """
    Parses the median keyed time and the data size.
    ([data_size], [time_total])
    """
    with open(filename) as f:
        lines = f.read().split('\n')
    xy_map = {}
    data_size = None
    key_index = None
    exitcode_index = None
    data = None

    for line in lines:
        line = line.strip()
        if 'Data Size' in line:
            data_size = int(line[11:-1])
            data = []
            continue
        if data_key in line:
            keys = line.split()
            for i in range(len(keys)):
                if keys[i] == data_key:
                    key_index = i
                elif keys[i] == 'exitcode':
                    exitcode_index = i
            continue
        if key_index is None:
            continue
        if line == '' or '***' in line:
            # Done reading data for this data_size
            if len(data) > 0:
                if data_size not in xy_map:
                    xy_map[data_size] = []
                xy_map[data_size].extend(data)
                if len(xy_map[data_size]) > NUM_TRIALS:
                    # truncated = len(xy_map[data_size]) - NUM_TRIALS
                    # print(f'{data_size}k truncating {truncated} points')
                    xy_map[data_size] = xy_map[data_size][:NUM_TRIALS]
            data_size = None
            key_index = None
            data = None
        else:
            # Read another data point for this data_size
            line = line.split()
            if exitcode_index is not None and int(line[exitcode_index]) != 0:
                continue
            data.append(float(line[key_index]))
    print(filename)

    xs = []
    ys = []
    for data_size in xy_map:
        if data_size in TARGET_XS:
            xs.append(data_size)
    xs.sort()
    if len(xs) != len(TARGET_XS):
        missing_xs = []
        for x in TARGET_XS:
            if x not in xs:
                missing_xs.append(x)
        print(f'missing {len(missing_xs)} xs: {missing_xs}')
    try:
        for i in range(len(xs)):
            x = xs[i]
            y = xy_map[x]
            if len(y) < NUM_TRIALS:
                missing = NUM_TRIALS - len(y)
                print(f'{x}k missing {missing}/{NUM_TRIALS}')
            xs[i] /= 1000.
            ys.append(DataPoint(y, normalize=xs[i] if normalize else None))
    except:
        import pdb; pdb.set_trace()
    return (xs, ys)

def get_filename(loss, cc, http):
    """
    Args:
    - loss: <number>
    - cc: reno, cubic
    - http: tcp, quic, pep
    """
    return '../results/{}/loss{}p/{}/{}.txt'.format(DATE, loss, cc, http)

def plot_graph(loss, cc, pdf,
               data_key='time_total',
               http_versions=['tcp-tso', 'pep-tso', 'quic'],
               use_median=True,
               normalize=True):
    data = {}
    for http_version in http_versions:
        filename = get_filename(loss, cc, http_version)
        if not path.exists(filename):
            print('Path does not exist: {}'.format(filename))
            continue
        try:
            data[http_version] = parse_data(filename, normalize,
                                            data_key=data_key)
        except Exception as e:
            print('Error parsing: {}'.format(filename))
            print(e)
    plt.clf()
    for (i, label) in enumerate(http_versions):
        if label not in data:
            continue
        (xs, ys_raw) = data[label]
        if use_median:
            ys = [y.p50 for y in ys_raw]
            yerr_lower = [y.p50 - y.p25 for y in ys_raw]
            yerr_upper = [y.p75 - y.p50 for y in ys_raw]
            plt.errorbar(xs, ys, yerr=(yerr_lower, yerr_upper), capsize=5,
                label=label, marker=MARKERS[i])
        else:
            ys = [y.avg for y in ys_raw]
            yerr = [y.stdev if y.stdev is not None else 0 for y in ys_raw]
            plt.errorbar(xs, ys, yerr=yerr, label=label, marker=MARKERS[i])
    plt.xlabel('Data Size (MB)')
    if normalize:
        plt.ylabel('{} tput (MB/s)'.format(data_key))
    else:
        plt.ylabel('{} (s)'.format(data_key))
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.4), ncol=3)
    statistic = 'median' if use_median else 'mean'
    plt.title('{} {} {}% loss'.format(statistic, cc, loss))
    if pdf is not None:
        save_pdf(pdf)

for cc in ['cubic']:
    for loss in [1, 2, 5]:
        plot_graph(loss=loss, cc=cc, pdf='median_{}_loss{}p.pdf'.format(cc, loss), use_median=True)
        plot_graph(loss=loss, cc=cc, pdf='mean_{}_loss{}p.pdf'.format(cc, loss), use_median=False)
