# NSDI '24 Experiments

Each figure in the paper has a corresponding script. The script parses log files
in an output directory and plots the data. If the log files are missing data,
the script executes the experiments and adds the data to the log files.

## Dependencies

Setup a Python virtual environment and install plotting dependencies.

```
python3 -m venv env
source env/bin/activate
pip install -r requirements.txt
```

Add `--logdir $HOME/sidecar/nsdi --execute` to everything.

```
python figures/fig2_microbenchmarks.py
python figures/fig4a_pep_emulation.py --marquee [-t 1]
python figures/fig4b_low_latency_media.py [-t 1]
python figures/fig4c_ack_reduction.py --marquee
python figures/fig5_baseline_bar.py --legend 0 [-t 1]
python figures/fig6_fairness.py --legend 0 [-t 1]
```
