# HTTP/3 file upload application

The client makes an HTTP/3 POST request to the server.

## Overview

The `libcurl` client integration mainly passes command-line options related to
QUIC and sidekicks from the `sidecurl` wrapper to the QUIC implementation.
The `sidecurl` wrapper also discovers sidekick proxies and receives quACKs.

The `quiche` client integration processes incoming quACKs and uses the
information in quACKs to influence QUIC, the base protocol.

The `nginx` server integration is based off an [unofficial patch](https://github.com/cloudflare/quiche/tree/master/nginx)
from Cloudflare that provides support for HTTP/3.
It uses [our fork of quiche](https://github.com/ygina/quiche/tree/sidecar)
mainly to implement a [QUIC extension](https://datatracker.ietf.org/doc/html/draft-ietf-quic-ack-frequency)
for changing an endpoint's acknowledgement frequency. The `nginx` webserver
forwards data from POST requests to a separate Python webserver to ensure the
data is collated and read.

## Real-World Experiment

Generate the data to send.

```console
client$ ./gen_data.sh 1M
client$ ./gen_data.sh 10M
client$ ./gen_data.sh 50M
```

Set the environment variables at the top of each script based on your
experimental setting.

Run the experiment, and parse the total time from the client for each request.
You do not need to restart the server nor proxy between trials.

```console
server$ ./start_nginx.sh
proxy$ ./start_proxy.sh
client$ ./run_client.sh [1M|10M|50M] [base|quack]
```
