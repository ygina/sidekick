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
