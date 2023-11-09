# Sidecar
```
 ________                                  ________                   ________
|        | h1-eth0                r1-eth0 |   r1   | r1-eth1 h2-eth0 |   h2   |
|        |<-------------------------------|--------|- - - - - - - - -|        |
|   h1   |                                |   ↑    |                 |   ↑    |
|sidecar |                                |sidecar |- - - - - - - - >|sidecar |
|________|                                |________|                 |________|
Data receiver                               Proxy                   Data sender
                                         QuACK sender            QuACK receiver
 10.0.1.10                            10.0.1.1   10.0.2.1             10.0.2.10
```

The following command is run from the `sidecar/` directory. Start the mininet
instance, which sets up the topology above and runs an NGINX/Python webserver
on h1:
```
$ sudo -E python3 mininet/net.py
```

The client POSTs an HTTP request with a payload of the specified size to the
webserver. Run an HTTP/1.1 or HTTP/3 (QUIC) client on h2 from the mininet CLI:
```
> h2 python3 mininet/client.py -n 100k --http 1 --trials 1
```

## Experiments

Setup the Python virtual environment for plotting data:

```
$ cd $SIDECAR_HOME/figures
$ virtualenv env
$ source env/bin/activate
$ pip3 install -r requirements.txt
```

### Figure 4 Baseline Bar Graphs

Check the baseline experiment, using a data size such as `10M`:

* pep: `sudo -E python3 mininet/net.py -t 1 --benchmark tcp --pep -n 10M`
* quack: `sudo -E python3 mininet/net.py -t 1 --benchmark quic -s 2ms --quack-reset -n 10M`
* tcp: `sudo -E python3 mininet/net.py -t 1 --benchmark tcp -n 10M`
* quack: `sudo -E python3 mininet/net.py -t 1 --benchmark quic -n 10M`

The goodput is 10MB divided by the total time of the request. To change
the loss percentage on the near subpath, use the `--loss2` parameter.
To plot the graph (and execute the data points), run `python3 baseline_bar.py -t 1 --execute`.

### Figure 5 Baseline Line Graphs

```
python3 data_size_vs_total_time.py --mean --median -t 1 --execute
```

### Figure 6 Congestion Control Behavior

We have to recompile `quiche` to log congestion window updates because
that is how we gather data points for QUIC. This just adds the `cwnd_log`
feature to the compilation command. For TCP, we use `ss`.

```
$ cd $SIDECAR_HOME/quiche
$ cargo build --package quiche --release --features ffi,pkg-config-meta,qlog,cwnd_log
```

Run a single instance of each category that lasts at least 30 seconds, and plot.

```
python3 time_vs_cwnd.py --max-x 30 --quic-n 35M --quack-n 35M --tcp 35 --loss 0 --execute
python3 time_vs_cwnd.py --max-x 30 --quic-n 5M --quack-n 35M --tcp 35 --loss 1 --execute
```

Make sure to recompile `quiche` for other experiments so you don't
log like crazy:

```
$ cd $SIDECAR_HOME/quiche
$ make sidecar
```

### Figure 7 Retransmission Behavior

```
python3 loss_vs_tput.py --max-x 800 -n 25M -t 1 --execute
```

### Figure 8 Multiflow Fairness

```
python3 multiflow.py -n 60M --max-x 60 --execute loss1p
python3 multiflow.py -n 60M --max-x 60 --execute loss0p
```

### Table 3 Analyzing Different Bit Widths


