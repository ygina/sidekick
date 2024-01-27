# Low-latency media application

Simple server and client in Rust for streaming low-latency media.

## Real-World Experiment

Set the environment variables at the top of each script based on your
experimental setting.

Run the experiment. The de-jitter buffer latencies should be logged to an output
file, whose path is an argument to the server script.
You do not need to restart the server nor proxy between trials.

```console
server$ ./start_server.sh
proxy$ ./start_proxy.sh
client$ ./run_client.sh [base|quack]
```
