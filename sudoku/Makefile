
all: check test

check:
	cargo check

build:
	cargo build

test:
	cargo test

bench:
	set RUST_TEST_THREADS=1
	cargo +nightly bench --features bench
