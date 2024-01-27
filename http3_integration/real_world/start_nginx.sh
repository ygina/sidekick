#!/bin/bash
sudo kill $(pidof nginx)
sudo nginx -c $HOME/sidecar/http3_integration/webserver/nginx.conf
python3 $HOME/sidecar/http3_integration/webserver/server.py
sudo kill $(pidof nginx)
