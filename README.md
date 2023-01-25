# fixhang
this branch attempts to fix a hanging issue in the mininet setup.

to trigger the early hang:

    $ cp webserver/server.original.py webserver/server.py
    $ sudo python3 mininet/net.py
    > h2 bash trigger_hang.sh
    (hang around 117)

to fix the hang...

    $ cp webserver/server.fixed.py webserver/server.py
    $ sudo python3 mininet/net.py
    > h2 bash trigger_hang.sh
    (completes all 1000 transfers)

the underlying issue appears to be the same as
https://github.com/mininet/mininet/issues/519#issuecomment-102198929
namely, the original server process was logging requests to stderr, which
mininet was reading from via a pipe. but if the server process is sent to the
background, mininet won't empty the buffer until the next time a command is run
on that server. this can cause the server to hang if, before that time, it
produces enough output to fill the pipe and hence block future reads.

a simpler solution might be to just pipe all output from the website into
/dev/null or a log file, if that info appears useful.

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

