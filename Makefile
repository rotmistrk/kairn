PREFIX ?= /usr/local
LOCAL_PREFIX ?= $(HOME)/.local

# ── IMPORTANT ───────────────────────────────────────────
# NEVER use `cargo install`. Use `make install-local` only.
# All binaries go to ~/.local/bin. The purge-cargo-bin target
# removes any stale copies from ~/.cargo/bin on every install.
# ─────────────────────────────────────────────────────────

.PHONY: all check lint clippy test test-fast fmt clean build release run \
        install-local uninstall-local purge-cargo-bin \
        install-kairn \
	verify setup check-hooks

MCP_LINT_DIR ?= $(HOME)/Workplace/mcp-lint

# ── Setup (run once per clone) ──────────────────────────

setup:
	@git config core.hooksPath hooks
	@echo "✅ Pre-commit hook enabled (hooks/pre-commit)"
	@if ! command -v mcp-lint-cli >/dev/null 2>&1; then \
		if [ -d "$(MCP_LINT_DIR)" ]; then \
			echo "  Installing mcp-lint-cli..."; \
			$(MAKE) -C $(MCP_LINT_DIR) install; \
		else \
			echo "  ⚠ mcp-lint-cli not found and $(MCP_LINT_DIR) missing"; \
			echo "    Clone: git clone git@github.com:rotmistrk/mcp-lint.git $(MCP_LINT_DIR)"; \
			exit 1; \
		fi; \
	else \
		echo "  ✓ mcp-lint-cli already installed"; \
	fi

check-hooks:
	@if [ "$$(git config core.hooksPath)" != "hooks" ]; then \
		echo "❌ Pre-commit hook not configured. Run: make setup"; exit 1; \
	fi

# ── Full cycle ──────────────────────────────────────────

all: fmt check lint test
	@echo "✅ All checks passed"

verify: fmt clippy test
	@echo "✅ Verification passed"

# ── Build ───────────────────────────────────────────────

BINARY := target/release/kairn
SOURCES := $(shell find src -name '*.rs') \
           $(if $(wildcard ../txv),$(shell find ../txv -name '*.rs' -not -path '*/target/*')) \
           Cargo.toml Cargo.lock

check: check-hooks
	cargo check --workspace

build: check-hooks
	cargo build --workspace

release: check-hooks $(BINARY)

$(BINARY): $(SOURCES)
	cargo build --workspace --release

run: $(BINARY)
	./$(BINARY)

# ── Quality ─────────────────────────────────────────────

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace -- -D warnings

# Fast tests: unit + lib only (for install gate)
test-fast: check-hooks
	cargo test --lib --workspace

# Full tests: all integration tests
test:
	@bash -c '\
	cargo test --workspace 2>&1 | tee /tmp/kairn-test-output.txt; \
	STATUS=$${PIPESTATUS[0]}; \
	if [ $$STATUS -ne 0 ]; then \
		echo "❌ Tests failed (exit code $$STATUS)"; \
		exit 1; \
	fi; \
	if grep -qE "[1-9][0-9]* ignored" /tmp/kairn-test-output.txt; then \
		echo "❌ FATAL: tests were skipped/ignored — all tests must run"; \
		exit 1; \
	fi; \
	echo "  ✅ All tests passed, none ignored"'

lint: clippy

# ── Install (all to ~/.local/bin) ───────────────────────

install-local: sync-deps test-fast purge-cargo-bin install-kairn install-rusticle-lsp 
	@echo "✅ Installed rusticle, rusticle-lsp, kairn, and demos to $(LOCAL_PREFIX)"

# Pull local dependency overrides (txv) if present, or update git dep
sync-deps:
	@if [ -d ../txv/.git ]; then \
		echo "  syncing txv (local)..."; \
		git -C ../txv pull --ff-only || true; \
	else \
		echo "  updating txv from git..."; \
		cargo update -p txv-core -p txv-widgets -p txv-render 2>/dev/null || true; \
	fi

# Remove stale copies from ~/.cargo/bin that shadow ~/.local/bin
purge-cargo-bin:
	@rm -f $(HOME)/.cargo/bin/kairn $(HOME)/.cargo/bin/rusticle 
	@echo "  🧹 Removed stale binaries from ~/.cargo/bin (if any)"

install-kairn: $(BINARY)
	install -d $(LOCAL_PREFIX)/bin
	install -m 755 $(BINARY) $(LOCAL_PREFIX)/bin/kairn
	@echo "  ✅ kairn → $(LOCAL_PREFIX)/bin/kairn"

install-rusticle-lsp: $(BINARY)
	install -d $(LOCAL_PREFIX)/bin
	install -m 755 target/release/rusticle-lsp $(LOCAL_PREFIX)/bin/rusticle-lsp
	@echo "  ✅ rusticle-lsp → $(LOCAL_PREFIX)/bin/rusticle-lsp"

# ── Uninstall ───────────────────────────────────────────

uninstall-local:
	rm -f $(LOCAL_PREFIX)/bin/kairn
	@echo "✅ Uninstalled"

# ── Clean ───────────────────────────────────────────────

clean:
	cargo clean

# ── Convenience ─────────────────────────────────────────

# Run rusticle REPL
repl:
	cargo run -p rusticle

# Run a rusticle script
run-script:
	@test -n "$(SCRIPT)" || (echo "Usage: make run-script SCRIPT=path.tcl" && exit 1)
	cargo run -p rusticle -- $(SCRIPT)

# Run a rusticle-tk app
