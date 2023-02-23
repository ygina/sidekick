#!/bin/bash
export SIDECAR_HOME=$HOME/sidecar

# exit if any errors
set -e

# git submodules
cd $SIDECAR_HOME
git submodule init
git submodule update
cd $SIDECAR_HOME/quiche
git submodule init
git submodule update

# Linux dependencies
sudo apt-get update -y
sudo apt-get install -y texlive # pari
sudo apt-get install -y autoconf libtool  # curl
sudo apt-get install -y cmake libpcre3 libpcre3-dev zlib1g zlib1g-dev libssl-dev  # nginx
sudo apt-get install -y libnfnetlink-dev  # pepsal
sudo apt-get install -y mininet python3-pip  # mininet

# mininet
pip3 install mininet

# rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  # hit 1
source "$HOME/.cargo/env"

# Download external dependencies
cd $SIDECAR_HOME/deps
curl -O https://nginx.org/download/nginx-1.16.1.tar.gz
tar xvzf nginx-1.16.1.tar.gz
wget https://pari.math.u-bordeaux.fr/pub/pari/unix/pari-2.15.2.tar.gz
tar xvzf pari-2.15.2.tar.gz
git clone git@github.com:viveris/pepsal.git
rm nginx-1.16.1.tar.gz pari-2.15.2.tar.gz

