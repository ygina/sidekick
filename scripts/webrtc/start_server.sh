#!/bin/bash
export RUST_LOG=debug
$HOME/sidecar/target/release/webrtc_server --port 5201 --rtt 110 --loop

