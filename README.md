# Sidecar
```
 ________                                  ________                   ________
|        | h1-eth0                r1-eth0 |   r1   | r1-eth1 h2-eth0 |   h2   |
|        |<-------------------------------|--------|- - - - - - - - -|        |
|   h1   |                                |   ↑    |                 |   ↑    |
|sidecar |                                |sidecar |- - - - - - - - >|sidecar |
|________|                                |________|                 |________|
Data receiver                               Proxy                   Data sender
                                         QuACK sender            QuACK receiver
 10.0.1.10                            10.0.1.1   10.0.2.1             10.0.2.10
```

The following command is run from the `sidecar/` directory. Start the mininet
instance, which sets up the topology above and runs an NGINX/Python webserver
on h1:
```
$ sudo -E python3 mininet/net.py
```

The client POSTs an HTTP request with a payload of the specified size to the
webserver. Run an HTTP/1.1 or HTTP/3 (QUIC) client on h2 from the mininet CLI:
```
> h2 python3 mininet/client.py -n 100k --http 1 --trials 1
```

## Dependencies

This assumes you have the following repositories in your home directory and
are on the correct branches:

```
$ git clone git@github.com:ygina/sidecar.git
$ git clone --recursive git@github.com:ygina/quiche.git
$ git clone https://github.com/curl/curl
$ curl -O https://nginx.org/download/nginx-1.16.1.tar.gz
$ tar xvzf nginx-1.16.1.tar.gz
quiche$ git checkout sidecar-v2
curl$ git checkout 2masot-sidecar
```

Build the nginx webserver with the HTTP/3 patch.
([source](https://github.com/ygina/quiche/tree/master/nginx))

```
$ sudo apt-get install libpcre3 libpcre3-dev zlib1g zlib1g-dev libssl-dev -y
nginx-1.16.1$ patch -p01 < ../quiche/nginx/nginx-1.16.patch
nginx-1.16.1$ ./configure                                 \
       --prefix=$PWD                           \
       --build="quiche-$(git --git-dir=$HOME/quiche/.git rev-parse --short HEAD)" \
       --with-http_ssl_module                  \
       --with-http_v2_module                   \
       --with-http_v3_module                   \
       --with-openssl=$HOME/quiche/quiche/deps/boringssl \
       --with-quiche=$HOME/quiche
nginx-1.16.1$ make
$ sudo ln -s $HOME/nginx-1.16.1/objs/nginx /usr/bin/nginx
```

Build and install curl (and sidecurl) with the HTTP/3 patch.

```
$ sudo apt install autoconf libtool
quiche$ make sidecar
quiche$ mkdir quiche/deps/boringssl/src/lib
quiche$ ln -vnf $(find target/release -name libcrypto.a -o -name libssl.a) quiche/deps/boringssl/src/lib/
curl$ autoreconf -fi
curl$ ./configure LDFLAGS="-Wl,-rpath,$HOME/quiche/target/release" --with-openssl=$HOME/quiche/quiche/deps/boringssl/src --with-quiche=$HOME/quiche/target/release
curl$ make -j4
curl$ cd sidecurl
sidecurl$ make
$ sudo ln -s $HOME/curl/sidecurl/sidecurl /usr/bin/sidecurl
```

Install the TCP performance-enhancing proxy,
[pepsal](https://github.com/viveris/pepsal).

```
$ git clone git@github.com:viveris/pepsal.git
$ sudo apt-get install -y libnfnetlink-dev
$ autoupdate
$ autoreconf --install
$ autoconf
$ ./configure
$ make
$ sudo make install
```

_(Optional: Install [PARI](http://pari.math.u-bordeaux.fr/download.html) for
factoring polynomials in finite fields.)_

```
$ wget https://pari.math.u-bordeaux.fr/pub/pari/unix/pari-2.15.2.tar.gz
$ tar xvf pari-2.15.2.tar.gz
$ sudo apt-get install -y texlive
pari-2.15.2$ ./Configure
pari-2.15.2$ make all
pari-2.15.2$ sudo make install
```

Check that `nginx`, `sidecurl`, and `pepsal` are on your path.

## Baselines

Check the baseline experiment, using a data size such as `10M`:

* pep: `sudo -E python3 mininet/net.py -t 1 --benchmark tcp --pep -n <DATA_SIZE>`
* quack: `sudo -E python3 mininet/net.py -t 1 --benchmark quic -s 2ms --quack-reset -n <DATA_SIZE>`
* tcp: `sudo -E python3 mininet/net.py -t 1 --benchmark tcp -n <DATA_SIZE>`
* quack: `sudo -E python3 mininet/net.py -t 1 --benchmark quic -n <DATA_SIZE>`
