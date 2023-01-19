import sys
import subprocess
import statistics

KEY = 'time_total'

if __name__ == '__main__':
    cmd = ['python3', 'mininet/net.py'] + sys.argv[1:]
    p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    key_indexes = {}
    data = []
    for line in p.stdout:
        sys.stdout.buffer.write(line)
        sys.stdout.buffer.flush()
        line = line.decode().strip()

        # Parse header
        if line.startswith('time_connect'):
            line = line.split()
            for i in range(len(line)):
                key_indexes[line[i]] = i
            continue

        # Header not parsed yet
        if len(key_indexes) == 0:
            continue

        # No more data
        if line == '' or '***' in line or '/tmp' in line:
            key_indexes = {}
            continue

        # Parse data
        data.append(float(line.split()[key_indexes[KEY]]))

    print(data)
    print('Median: {:.4f}'.format(statistics.median(data)))
    print('Mean: {:.4f}'.format(statistics.mean(data)))
    if len(data) > 1:
        print('Stdev: {:.4f}'.format(statistics.stdev(data)))
