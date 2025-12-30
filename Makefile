.PHONY: run build release clean test check fmt lint

# Development
run:
	cargo run

build:
	cargo build

release:
	cargo build --release

clean:
	cargo clean

# Quality
check:
	cargo check

fmt:
	cargo fmt

lint:
	cargo clippy

# Release tags
tag:
	@read -p "Version (e.g., 0.1.0): " version; \
	git tag -a "v$$version" -m "Release v$$version"; \
	echo "Created tag v$$version. Push with: git push origin v$$version"

# Cross-compile targets
build-windows:
	cargo build --release --target x86_64-pc-windows-msvc

build-linux:
	cargo build --release --target x86_64-unknown-linux-gnu

build-macos-intel:
	cargo build --release --target x86_64-apple-darwin

build-macos-arm:
	cargo build --release --target aarch64-apple-darwin

build-all: build-macos-intel build-macos-arm
	@echo "Built for all macOS targets"
