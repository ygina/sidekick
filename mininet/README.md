# Running a single experiment

The file `mininet/main.py` is the entrypoint into running a single experiment
in a two-hop network topology, where a data sender and data receiver exists on
either end, and a (possibly performance-enhancing) proxy exists in the middle.

```
sudo -E python3 mininet/main.py -t 1 -n 10M [quic|quack|tcp|pep]
```
