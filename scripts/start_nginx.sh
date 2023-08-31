#!/bin/bash
sudo kill $(pidof nginx)
sudo nginx -c $HOME/sidecar/webserver/nginx.conf
python3 $HOME/sidecar/webserver/server.py
sudo kill $(pidof nginx)

