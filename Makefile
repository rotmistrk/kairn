PREFIX ?= /usr/local
LOCAL_PREFIX ?= $(HOME)/.local
BINARY = kairn

.PHONY: all check lint clippy style test fmt clean build release \
        install install-local uninstall uninstall-local

# Full cycle: format, compile check, clippy, lint, test
all: fmt check lint test
	@echo "✅ All checks passed"

# Compile check (fast feedback)
check:
	cargo check

# Clippy (includes unwrap/expect deny)
lint: clippy style

clippy:
	cargo clippy -- -D warnings

# Custom lint rules via check_file MCP tool (run from kiro)
style:
	@echo "Run 'check_file' on modified .rs files via kiro"

# Run all tests
test:
	cargo test

# Format
fmt:
	cargo fmt

# Debug build
build:
	cargo build

# Release build
release:
	cargo build --release

# Install to /usr/local/bin (requires sudo)
install: release
	install -d $(PREFIX)/bin
	install -m 755 target/release/$(BINARY) $(PREFIX)/bin/$(BINARY)
	@echo "✅ Installed to $(PREFIX)/bin/$(BINARY)"

# Install to ~/.local/bin (no sudo needed)
install-local: release
	install -d $(LOCAL_PREFIX)/bin
	install -m 755 target/release/$(BINARY) $(LOCAL_PREFIX)/bin/$(BINARY)
	@echo "✅ Installed to $(LOCAL_PREFIX)/bin/$(BINARY)"

# Uninstall from /usr/local/bin
uninstall:
	rm -f $(PREFIX)/bin/$(BINARY)
	@echo "✅ Removed $(PREFIX)/bin/$(BINARY)"

# Uninstall from ~/.local/bin
uninstall-local:
	rm -f $(LOCAL_PREFIX)/bin/$(BINARY)
	@echo "✅ Removed $(LOCAL_PREFIX)/bin/$(BINARY)"

# Clean
clean:
	cargo clean
