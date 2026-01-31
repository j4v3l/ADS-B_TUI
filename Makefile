CARGO ?= cargo

.PHONY: build build-release run run-insecure fmt clippy test clean

build:
	$(CARGO) build

build-release:
	$(CARGO) build --release

run:
	$(CARGO) run --

run-insecure:
	$(CARGO) run -- --insecure

fmt:
	$(CARGO) fmt

clippy:
	$(CARGO) clippy -- -D warnings

test:
	$(CARGO) test

clean:
	$(CARGO) clean
