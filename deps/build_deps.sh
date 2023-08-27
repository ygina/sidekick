#!/bin/bash
if [ $# -ne 1 ]; then
	echo "USAGE: $0 [all|0|1|2|3|4|5|6]"
	echo "0 = nginx"
	echo "1 = pari"
	echo "2 = quiche"
	echo "3 = curl"
	echo "4 = sidecurl"
	echo "5 = pepsal"
	echo "6 = sidecar"	
	exit 1
fi

export SIDECAR_HOME=$HOME/sidecar

build_nginx () {
cd $SIDECAR_HOME/deps/nginx-1.16.1
mkdir -p logs
patch -N -r- -p01 < $SIDECAR_HOME/quiche-nginx/nginx/nginx-1.16.patch
./configure                                 \
   --prefix=$PWD                           \
   --build="quiche-$(git --git-dir=$SIDECAR_HOME/quiche-nginx/.git rev-parse --short HEAD)" \
   --with-http_ssl_module                  \
   --with-http_v2_module                   \
   --with-http_v3_module                   \
   --with-openssl=$SIDECAR_HOME/quiche-nginx/quiche/deps/boringssl \
   --with-quiche=$SIDECAR_HOME/quiche-nginx
make -j$(nproc)
sudo ln -f -s $(pwd)/objs/nginx /usr/bin/nginx
}

build_pari () {
cd $SIDECAR_HOME/deps/pari-2.15.2
./Configure
make -j$(nproc) all
sudo make install
sudo ldconfig
}

build_quiche () {
cd $SIDECAR_HOME/quiche
make sidecar
mkdir -p quiche/deps/boringssl/src/lib
ln -f -vnf $(find target/release -name libcrypto.a -o -name libssl.a) quiche/deps/boringssl/src/lib/
}

build_curl () {
cd $SIDECAR_HOME/curl
autoreconf -fi
./configure LDFLAGS="-Wl,-rpath,$SIDECAR_HOME/quiche/target/release" \
        --with-openssl=$SIDECAR_HOME/quiche/quiche/deps/boringssl/src \
        --with-quiche=$SIDECAR_HOME/quiche/target/release
make -j$(nproc)
}

build_sidecurl () {
cd $SIDECAR_HOME/curl/sidecurl
make
sudo ln -f -s $SIDECAR_HOME/curl/sidecurl/sidecurl /usr/bin/sidecurl
}

build_pepsal () {
cd $SIDECAR_HOME/deps/pepsal
autoupdate
autoreconf --install
autoconf
./configure
make
sudo make install
}

build_sidecar () {
cd $SIDECAR_HOME
cargo build --release --features libpari
}

if [ $1 == "all" ]; then
	build_nginx
	build_pari
	build_quiche
	build_curl
	build_sidecurl
	build_pepsal
	build_sidecar
elif [ $1 -eq 0 ]; then
	build_nginx
elif [ $1 -eq 1 ]; then
	build_pari
elif [ $1 -eq 2 ]; then
	build_quiche
elif [ $1 -eq 3 ]; then
	build_curl
elif [ $1 -eq 4 ]; then
	build_sidecurl
elif [ $1 -eq 5 ]; then
	build_pepsal
elif [ $1 -eq 6 ]; then
	build_sidecar
else
	echo "USAGE: $0 [all|0|1|2|3|4|5|6]"
	echo "0 = nginx"
	echo "1 = pari"
	echo "2 = quiche"
	echo "3 = curl"
	echo "4 = sidecurl"
	echo "5 = pepsal"
	echo "6 = sidecar"	
fi

