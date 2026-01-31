# Justfile for adsb-tui
# A command runner for common development tasks

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode
build-release:
    cargo build --release

# Run the application
run:
    cargo run --

# Run the application with insecure TLS
run-insecure:
    cargo run -- --insecure

# Format the code
fmt:
    cargo fmt

# Run clippy with warnings as errors
clippy:
    cargo clippy -- -D warnings

# Run tests
test:
    cargo test

# Clean build artifacts
clean:
    cargo clean

# Run all checks (fmt, clippy, test)
check: fmt clippy test

# Build and run
build-run: build run

# Build release and run
release-run: build-release
    ./target/release/adsb-tui

# Update dependencies
update:
    cargo update

# Show project info
info:
    @echo "adsb-tui - ADS-B Terminal User Interface"
    @echo "Version: $(shell cargo pkgid | cut -d# -f2 | cut -d: -f2)"
    @echo "Rust version: $(shell rustc --version)"
    @echo "Cargo version: $(shell cargo --version)"

# Install the binary
install: build-release
    cargo install --path .

# Generate documentation
doc:
    cargo doc --open

# Watch for changes and rebuild
watch:
    cargo watch -x build

# Run with verbose output
run-verbose:
    RUST_LOG=debug cargo run --

# Check for unused dependencies
udeps:
    cargo +nightly udeps

# Audit dependencies for security issues
audit:
    cargo audit

# Format and check
fmt-check:
    cargo fmt --check

# Run tests with coverage (requires tarpaulin)
test-coverage:
    cargo tarpaulin --ignore-tests

# Clean and rebuild everything
rebuild: clean build