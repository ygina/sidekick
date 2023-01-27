# Imports
from matplotlib import pyplot as plt
from matplotlib.backends.backend_pdf import PdfPages
from matplotlib.transforms import Bbox
import seaborn as sns

# Plot markers.
MARKERS = 'PXD*o^v<>'

# Configure graph styling library.
sns.set_style('ticks')
font = {
    'font.weight': 1000,
    'font.size': 15,
}
sns.set_style(font)
paper_rc = {
    'lines.linewidth': 2,
    'lines.markersize': 10,
}
sns.set_context("paper", font_scale=3,  rc=paper_rc)
plt.style.use('seaborn-v0_8-deep')


def save_pdf(output_filename, bbox_inches='tight'):
    if output_filename is not None:
        with PdfPages(output_filename) as pdf:
            pdf.savefig(bbox_inches=bbox_inches)
