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

# Run tests with coverage (requires cargo-llvm-cov)
coverage:
    RUSTUP_TOOLCHAIN=stable-aarch64-apple-darwin cargo llvm-cov --workspace

# Generate HTML coverage report (requires cargo-llvm-cov)
coverage-html:
    RUSTUP_TOOLCHAIN=stable-aarch64-apple-darwin cargo llvm-cov --workspace --html

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
    @echo "ADS-B TUI - Modern ADS-B Aircraft Tracking"
    @echo "Version: $(shell cargo pkgid | cut -d# -f2 | cut -d: -f2)"
    @echo "Rust version: $(shell rustc --version)"
    @echo "Cargo version: $(shell cargo --version)"
    @echo ""
    @echo "Useful commands:"
    @echo "  just build          - Build in debug mode"
    @echo "  just build-release  - Build in release mode"
    @echo "  just test           - Run tests"
    @echo "  just check          - Run all checks (fmt, clippy, test)"
    @echo "  just run            - Run the application"
    @echo "  just doc            - Generate documentation"
    @echo "  just clean          - Clean build artifacts"

# Install the binary
install: build-release
    cargo install --path .

# Generate documentation
doc:
    cargo doc --open --no-deps

# Watch for changes and rebuild
watch:
    cargo watch -x check

# Run with verbose output
run-verbose:
    RUST_LOG=debug cargo run --

# Check for unused dependencies (requires cargo-udeps)
udeps:
    cargo +nightly udeps --all-targets

# Audit dependencies for security issues (requires cargo-audit)
audit:
    cargo audit

# Format and check
fmt-check:
    cargo fmt --all --check

# Run tests with coverage (requires cargo-tarpaulin)
test-coverage:
    cargo tarpaulin --ignore-tests --out Html

# Clean and rebuild everything
rebuild: clean build

# Run benchmarks (requires cargo-criterion)
bench:
    cargo bench

# Check minimum supported Rust version
msrv:
    cargo msrv verify

# Update dependencies
update-deps:
    cargo update
    cargo outdated

# Lint and check everything
lint: fmt clippy

# Full CI check (what runs in CI)
ci: fmt-check clippy test audit

# Release workflow
release: test audit build-release
    @echo "Release build complete. Binaries in target/release/"

# Development setup
setup-dev:
    cargo install cargo-watch
    cargo install cargo-edit
    cargo install cargo-audit
    cargo install cargo-outdated
    @echo "Development tools installed"
