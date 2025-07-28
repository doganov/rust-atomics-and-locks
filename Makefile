.PHONY: test test-plain test-loom test-miri

test: test-plain test-loom test-miri

test-plain:
	cargo test

test-loom:
	RUSTFLAGS="--cfg loom" cargo test --release

test-miri:
	cargo +nightly miri test
