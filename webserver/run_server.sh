#!/bin/bash
sudo kill $(cat /home/gina/nginx-1.16.1/logs/nginx.pid)
sudo nginx -c /home/gina/webserver/nginx.conf
python3 server.py
