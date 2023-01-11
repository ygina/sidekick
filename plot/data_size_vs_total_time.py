import os.path
import statistics
from os import path
from common import *

DATE = '010922'
NUM_TRIALS = 7
NUM_XS = 10

class DataPoint:
    def __init__(self, arr):
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

def parse_data(filename, data_key='time_total'):
    """
    Parses the median keyed time and the data size.
    ([data_size], [time_total])
    """
    with open(filename) as f:
        lines = f.read().split('\n')
    xs = []
    ys = []
    data_size = None
    key_index = None
    data = None
    count = 0

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
                    break 
            continue
        if key_index is None:
            continue
        if line == '' or '***' in line:
            # Done reading data for this data_size
            if len(data) > 0:
                xs.append(data_size)
                ys.append(DataPoint(data))
            count += len(data)
            data_size = None
            key_index = None
            data = None
        else:
            # Read another data point for this data_size
            data.append(float(line.split()[key_index]))            
    print('{}: missing data points = {}'.format(
        filename, count % (NUM_TRIALS * NUM_XS)))
    return (xs, ys)

def get_filename(loss, cc, http):
    """
    Args:
    - loss: <number>
    - cc: reno, cubic
    - http: tcp, quic, pep
    """
    return '../results/{}/loss{}p/{}/{}.txt'.format(DATE, loss, cc, http)

def plot_graph(loss, cc, pdf, data_key='time_total', http_versions=['tcp', 'pep', 'quic']):
    data = {}
    for http_version in http_versions:
        filename = get_filename(loss, cc, http_version)
        if not path.exists(filename):
            print('Path does not exist: {}'.format(filename))
            continue
        try:
            data[http_version] = parse_data(filename, data_key=data_key)
        except Exception as e:
            print('Error parsing: {}'.format(filename))
            print(e)
    plt.clf()
    for (i, label) in enumerate(http_versions):
        if label not in data:
            continue
        (xs, ys) = data[label]
        ys = [point.p50 for point in ys]
        plt.plot(xs, ys, label=label, marker=MARKERS[i])
    plt.xlabel('Data Size (kB)')
    plt.ylabel('{} (s)'.format(data_key))
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.4), ncol=3)
    plt.title('{} {}% loss'.format(cc, loss))
    if pdf is not None:
        save_pdf(pdf)

for cc in ['cubic', 'reno']:
    for loss in [1, 2, 5, 10]:
        plot_graph(loss=loss, cc=cc, pdf='{}_loss{}p.pdf'.format(cc, loss))
