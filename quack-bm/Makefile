build:
	cargo b --release

construct: build
	../target/release/quack-bm construct -t 20 -n 1000 -b 32 --dropped 20

decode: build
	../target/release/quack-bm decode -t 20 -n 1000 -b 32 --dropped 20
