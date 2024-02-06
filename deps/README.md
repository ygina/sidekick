# Dependencies

Install packaged dependencies and download external dependencies, assuming
the `sidekick` GitHub repository is located in `$HOME`.

```
./install_deps.sh
```

Activate Python virtual environments and Rust. Set the default Rust toolchain.

```
cd $HOME/sidekick/figures/
source env/bin/activate
pip install -r requirements.txt
source "$HOME/.cargo/env"
echo "source $HOME/sidekick/figures/env/bin/activate" >> ~/.bashrc  # optional
```

Build various dependencies.

```
cd $HOME/sidekick/deps
./build_deps.sh 0  # nginx (http3 server)
./build_deps.sh 1  # pari (quack "libpari" feature for modular factorization)
./build_deps.sh 2  # quiche (http3 client)
./build_deps.sh 3  # libcurl (http3 client)
./build_deps.sh 4  # sidecurl (http3 client)
./build_deps.sh 5  # pepsal (TCP PEP baseline)
./build_deps.sh 6  # sidecar (experimental binaries)
```

Check that `nginx`, `sidecurl`, and `pepsal` are on your path.
