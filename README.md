# fixhang
this branch attempts to fix a hanging issue in the mininet setup.

To trigger the early hang:

    $ sudo python3 mininet/net.py
    > h2 bash trigger_hang.sh
    (hangs around request #117)

# sidecar
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
$ sudo python3 mininet/net.py
```

The client POSTs an HTTP request with a payload of the specified size to the
webserver. Run an HTTP/1.1 or HTTP/3 (QUIC) client on h2 from the mininet CLI:
```
> h2 ../webserver/run_client.sh 1k 1
```

