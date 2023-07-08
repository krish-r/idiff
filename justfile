default:
	@just --list --unsorted

clean:
	cargo clean

check:
	cargo check

check-tests:
	cargo check --tests

clippy:
	cargo clippy

clippy-tests:
	cargo clippy --tests

build:
	cargo build

release:
	cargo build --release

run:
	cargo run

test:
	cargo test

insta-test:
	cargo insta test

insta-review:
	cargo insta test

