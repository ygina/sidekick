import os.path
from os import path
from common import *

def median(arr):
    assert len(arr) != 0
    arr.sort()
    mid = int(len(arr) / 2)
    if len(arr) % 2 == 1:
        return arr[mid]
    else:
        return (arr[mid] + arr[mid+1]) / 2.0

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
        if line == '':
            # Done reading data for this data_size
            xs.append(data_size)
            ys.append(median(data))
            data_size = None
            key_index = None
            data = None
        else:
            # Read another data point for this data_size
            data.append(float(line.split()[key_index]))            
    return (xs, ys)

def get_filename(cc, http):
    """
    Args:
    - cc: reno, cubic
    - http: tcp, quic, pep
    """
    return '../results/{}/{}.txt'.format(cc, http)

def plot_graph(cc, pdf, data_key='time_total', http_versions=['tcp', 'pep', 'quic']):
    data = {}
    for http_version in http_versions:
        filename = get_filename(cc, http_version)
        if not path.exists(filename):
            print('Path does not exist: {}'.format(filename))
            continue
        data[http_version] = parse_data(filename, data_key=data_key)
    plt.clf()
    for (i, label) in enumerate(http_versions):
        if label not in data:
            continue
        (x, y) = data[label]
        plt.plot(x, y, label=label, marker=MARKERS[i])
    plt.xlabel('Data Size (kB)')
    plt.ylabel('{} (s)'.format(data_key))
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.4), ncol=3)
    plt.title(pdf)
    if pdf is not None:
        save_pdf(pdf)

plot_graph(cc='cubic', pdf='cubic.pdf')
plot_graph(cc='reno', pdf='reno.pdf')
