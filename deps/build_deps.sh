#!/bin/bash
if [ $# -ne 1 ]; then
	echo "USAGE: $0 [all|0|1|2|3|4|5|6]"
	echo "0 = nginx"
	echo "1 = pari"
	echo "2 = quiche"
	echo "3 = libcurl"
	echo "4 = sidecurl"
	echo "5 = pepsal"
	echo "6 = sidekick"
	exit 1
fi

export SIDEKICK_HOME=$HOME/sidekick

build_nginx () {
cd $SIDEKICK_HOME/deps/nginx-1.16.1
mkdir -p logs
patch -N -r- -p01 < $SIDEKICK_HOME/deps/quiche-nginx/nginx/nginx-1.16.patch
sed -i 's\ffi"\ffi,power_sum"\g' auto/lib/quiche/make
cp $SIDEKICK_HOME/deps/ngx_http_v3_module* src/http/v3/
./configure                                 \
   --prefix=$PWD                           \
   --build="quiche-$(git --git-dir=$SIDEKICK_HOME/deps/quiche-nginx/.git rev-parse --short HEAD)" \
   --with-http_ssl_module                  \
   --with-http_v2_module                   \
   --with-http_v3_module                   \
   --with-openssl=$SIDEKICK_HOME/deps/quiche-nginx/quiche/deps/boringssl \
   --with-quiche=$SIDEKICK_HOME/deps/quiche-nginx
make -j$(nproc)
sudo ln -f -s $(pwd)/objs/nginx /usr/bin/nginx
}

build_pari () {
cd $SIDEKICK_HOME/deps/pari-2.15.2
./Configure
make -j$(nproc) all
sudo make install
sudo ldconfig
}

build_quiche () {
cd $SIDEKICK_HOME/http3_integration/quiche
make sidekick
mkdir -p quiche/deps/boringssl/src/lib
ln -f -vnf $(find target/release -name libcrypto.a -o -name libssl.a) quiche/deps/boringssl/src/lib/
}

build_libcurl () {
cd $SIDEKICK_HOME/http3_integration/curl
autoreconf -fi
./configure LDFLAGS="-Wl,-rpath,$SIDEKICK_HOME/http3_integration/quiche/target/release" \
        --with-openssl=$SIDEKICK_HOME/http3_integration/quiche/quiche/deps/boringssl/src \
        --with-quiche=$SIDEKICK_HOME/http3_integration/quiche/target/release
make -j$(nproc)
}

build_sidecurl () {
cd $SIDEKICK_HOME/http3_integration/curl/sidecurl
make
sudo ln -f -s $SIDEKICK_HOME/http3_integration/curl/sidecurl/sidecurl /usr/bin/sidecurl
}

build_pepsal () {
cd $SIDEKICK_HOME/deps/pepsal
autoupdate
autoreconf --install
autoconf
./configure
make
sudo make install
}

build_sidekick () {
cd $SIDEKICK_HOME
cargo build --release
cargo build --release --examples --all-features
}

if [ $1 == "all" ]; then
	build_nginx
	build_pari
	build_quiche
	build_libcurl
	build_sidecurl
	build_pepsal
	build_sidekick
elif [ $1 -eq 0 ]; then
	build_nginx
elif [ $1 -eq 1 ]; then
	build_pari
elif [ $1 -eq 2 ]; then
	build_quiche
elif [ $1 -eq 3 ]; then
	build_libcurl
elif [ $1 -eq 4 ]; then
	build_sidecurl
elif [ $1 -eq 5 ]; then
	build_pepsal
elif [ $1 -eq 6 ]; then
	build_sidekick
else
	echo "USAGE: $0 [all|0|1|2|3|4|5|6]"
	echo "0 = nginx"
	echo "1 = pari"
	echo "2 = quiche"
	echo "3 = curl"
	echo "4 = sidecurl"
	echo "5 = pepsal"
	echo "6 = sidekick"	
fi

