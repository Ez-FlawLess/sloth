.PHONY: test-miri run-miri test build run

# Run tests with miri (requires nightly)
# Suppress unused warnings during build
test-miri:
	RUSTFLAGS="-A unused" cargo +nightly miri test

# Run binary with miri (requires nightly)
run-miri:
	cargo +nightly miri run

# Regular test (uses stable toolchain)
test:
	cargo test

# Regular build (uses stable toolchain)
build:
	cargo build

# Regular run (uses stable toolchain)
run:
	cargo run
