import os.path
from os import path
from common import *

def median(arr):
    try:
        arr.sort()
        mid = int(len(arr) / 2)
        if len(arr) % 2 == 1:
            return arr[mid]
        else:
            return (arr[mid] + arr[mid-1]) / 2.0
    except:
        import pdb; pdb.set_trace()

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
                ys.append(median(data))
            count += len(data)
            data_size = None
            key_index = None
            data = None
        else:
            # Read another data point for this data_size
            data.append(float(line.split()[key_index]))            
    if count > 70:
        count /= 2
    print('{}: {}'.format(filename, 70 - count))
    return (xs, ys)

def get_filename(loss, cc, http):
    """
    Args:
    - loss: <number>
    - cc: reno, cubic
    - http: tcp, quic, pep
    """
    return '../results/loss{}p/{}/{}.txt'.format(loss, cc, http)

def plot_graph(loss, cc, pdf, data_key='time_total', http_versions=['tcp', 'pep', 'quic']):
    data = {}
    for http_version in http_versions:
        filename = get_filename(loss, cc, http_version)
        if not path.exists(filename):
            print('Path does not exist: {}'.format(filename))
            continue
        try:
            data[http_version] = parse_data(filename, data_key=data_key)
        except:
            print('Error parsing: {}'.format(filename))
    plt.clf()
    for (i, label) in enumerate(http_versions):
        if label not in data:
            continue
        (x, y) = data[label]
        plt.plot(x, y, label=label, marker=MARKERS[i])
    plt.xlabel('Data Size (kB)')
    plt.ylabel('{} (s)'.format(data_key))
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.4), ncol=3)
    plt.title('{} {}% loss'.format(cc, loss))
    if pdf is not None:
        save_pdf(pdf)

for cc in ['cubic', 'reno']:
    for loss in [1, 2, 5, 10]:
        plot_graph(loss=loss, cc=cc, pdf='{}_loss{}p.pdf'.format(cc, loss))
