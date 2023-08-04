import argparse
import subprocess
import os
import re
import sys
import os.path
import statistics
import math
from os import path
from collections import defaultdict
from common import *

WORKDIR = os.environ['HOME'] + '/sidecar'

def plot_graph(data, keys=['base', 'quack'], legend=True, pdf=None):
    plt.figure(figsize=(9, 6))
    for (i, key) in enumerate(keys):
        ys = [y / 1000000.0 for y in data[key]]
        plt.plot(range(101), ys, marker=MARKERS[i], label=key)
    plt.xlabel('Percentile')
    plt.ylabel('Latency (ms)')
    plt.xlim(0, 100)
    plt.ylim(0)
    if legend:
        plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.3), ncol=2)
    plt.title(pdf)
    if pdf:
        save_pdf(f'{WORKDIR}/plot/graphs/{pdf}')
    plt.clf()

if __name__ == '__main__':
    data = {}
    data['base'] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1668224, 2545020, 2567731, 2605140, 2703335, 2814757, 51795902, 51859531, 52746712, 52840537, 52845250, 52923285, 53077888, 54776323, 102061342, 102117476, 103062726, 103115459, 103126969, 103204672, 105076598, 152353668, 152363188, 152364961, 152390883, 152393023, 152397792, 152410288, 152432109, 155090442, 202718757, 252010432, 254627990, 302888344, 352411566, 353048094]
    data['quack'] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3991917, 4091838, 4956121, 4975895, 4988555, 5009217, 5011281, 5052052, 5089611, 15154229, 15175043]
    pdf = f'latencies_webrtc.pdf'
    plot_graph(data, pdf=pdf)
