#!/bin/bash
sudo kill $(pidof nginx)
sudo nginx -c $HOME/sidekick/http3_integration/webserver/nginx.conf
python3 $HOME/sidekick/http3_integration/webserver/server.py
sudo kill $(pidof nginx)
