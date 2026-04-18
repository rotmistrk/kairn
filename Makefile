.PHONY: check lint test all clean fmt

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
# For CI, use: find src -name '*.rs' -exec kiro-lint {} \;
style:
	@echo "Run 'check_file' on modified .rs files via kiro"

# Run all tests
test:
	cargo test

# Format
fmt:
	cargo fmt

# Clean
clean:
	cargo clean
