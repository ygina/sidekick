RUST_LOG ?= info

build:
	cargo b --release

construct: build
	RUST_LOG=$(RUST_LOG) ../target/release/quack-bm construct -t 20 -n 1000 -b 32 --dropped 20

decode: build
	RUST_LOG=$(RUST_LOG) ../target/release/quack-bm decode -t 20 -n 1000 -b 32 --dropped 20
