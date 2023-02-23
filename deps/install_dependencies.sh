#!/bin/bash
export SIDECAR_HOME=$HOME/sidecar

# exit if any errors
set -e

# Linux dependencies
sudo apt-get update -y
sudo apt-get install -y texlive # pari
sudo apt-get install -y autoconf libtool  # curl
sudo apt-get install -y cmake libpcre3 libpcre3-dev zlib1g zlib1g-dev libssl-dev  # nginx
sudo apt-get install -y libnfnetlink-dev  # pepsal
sudo apt-get install -y mininet python3-pip  # mininet

# rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  # hit 1
source "$HOME/.cargo/env"

# mininet
pip3 install mininet

# Download external dependencies
cd $SIDECAR_HOME/deps
curl -O https://nginx.org/download/nginx-1.16.1.tar.gz
tar xvzf nginx-1.16.1.tar.gz
wget https://pari.math.u-bordeaux.fr/pub/pari/unix/pari-2.15.2.tar.gz
tar xvzf pari-2.15.2.tar.gz
git clone git@github.com:viveris/pepsal.git
rm nginx-1.16.1.tar.gz pari-2.15.2.tar.gz

# nginx
cd $SIDECAR_HOME/deps/nginx-1.16.1
mkdir logs
patch -p01 < $SIDECAR_HOME/quiche/nginx/nginx-1.16.patch
./configure                                 \
   --prefix=$PWD                           \
   --build="quiche-$(git --git-dir=$SIDECAR_HOME/quiche/.git rev-parse --short HEAD)" \
   --with-http_ssl_module                  \
   --with-http_v2_module                   \
   --with-http_v3_module                   \
   --with-openssl=$SIDECAR_HOME/quiche/quiche/deps/boringssl \
   --with-quiche=$SIDECAR_HOME/quiche
make -j$(nproc)
sudo ln -s $(pwd)/objs/nginx /usr/bin/nginx

# pari
cd $SIDECAR_HOME/deps/pari-2.15.2
./Configure
make -j$(nproc) all
sudo make install

# quiche
cd $SIDECAR_HOME/quiche
make sidecar
mkdir quiche/deps/boringssl/src/lib
ln -vnf $(find target/release -name libcrypto.a -o -name libssl.a) quiche/deps/boringssl/src/lib/

# curl
cd $SIDECAR_HOME/curl
autoreconf -fi
./configure LDFLAGS="-Wl,-rpath,$SIDECAR_HOME/quiche/target/release" \
	--with-openssl=$SIDECAR_HOME/quiche/quiche/deps/boringssl/src \
	--with-quiche=$SIDECAR_HOME/quiche/target/release
make -j$(nproc)

# sidecurl
cd $SIDECAR_HOME/curl/sidecurl
make
sudo ln -s $SIDECAR_HOME/curl/sidecurl/sidecurl /usr/bin/sidecurl

# pepsal
cd $SIDECAR_HOME/deps/pepsal
autoupdate
autoreconf --install
autoconf
./configure
make
sudo make install

# sidecar
cd $SIDECAR_HOME
cargo build --release --features libpari

