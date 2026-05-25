PREFIX ?= /usr/local
LOCAL_PREFIX ?= $(HOME)/.local
TK_DEMO_DIR ?= $(LOCAL_PREFIX)/share/rusticle-tk/examples

# ── IMPORTANT ───────────────────────────────────────────
# NEVER use `cargo install`. Use `make install-local` only.
# All binaries go to ~/.local/bin. The purge-cargo-bin target
# removes any stale copies from ~/.cargo/bin on every install.
# ─────────────────────────────────────────────────────────

.PHONY: all check lint clippy test test-fast fmt clean build release run \
        install-local uninstall-local purge-cargo-bin \
        install-rusticle-tk install-kairn \
        install-demos verify setup check-hooks

# ── Setup (run once per clone) ──────────────────────────

setup:
	@git config core.hooksPath hooks
	@echo "✅ Pre-commit hook enabled (hooks/pre-commit)"

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

install-local: test-fast purge-cargo-bin install-rusticle-tk install-kairn install-rusticle-lsp install-demos
	@echo "✅ Installed rusticle, rusticle-tk, rusticle-lsp, kairn, and demos to $(LOCAL_PREFIX)"

# Remove stale copies from ~/.cargo/bin that shadow ~/.local/bin
purge-cargo-bin:
	@rm -f $(HOME)/.cargo/bin/kairn $(HOME)/.cargo/bin/rusticle $(HOME)/.cargo/bin/rusticle-tk
	@echo "  🧹 Removed stale binaries from ~/.cargo/bin (if any)"

install-rusticle-tk: $(BINARY)
	install -d $(LOCAL_PREFIX)/bin
	install -m 755 target/release/rusticle-tk $(LOCAL_PREFIX)/bin/rusticle-tk
	@echo "  ✅ rusticle-tk → $(LOCAL_PREFIX)/bin/rusticle-tk"

install-kairn: $(BINARY)
	install -d $(LOCAL_PREFIX)/bin
	install -m 755 $(BINARY) $(LOCAL_PREFIX)/bin/kairn
	@echo "  ✅ kairn → $(LOCAL_PREFIX)/bin/kairn"

install-rusticle-lsp: $(BINARY)
	install -d $(LOCAL_PREFIX)/bin
	install -m 755 target/release/rusticle-lsp $(LOCAL_PREFIX)/bin/rusticle-lsp
	@echo "  ✅ rusticle-lsp → $(LOCAL_PREFIX)/bin/rusticle-lsp"

install-demos:
	install -d $(TK_DEMO_DIR)
	install -m 644 rusticle-tk/examples/*.tcl $(TK_DEMO_DIR)/
	@echo "  ✅ rusticle-tk demos → $(TK_DEMO_DIR)/"

# ── Uninstall ───────────────────────────────────────────

uninstall-local:
	rm -f $(LOCAL_PREFIX)/bin/rusticle
	rm -f $(LOCAL_PREFIX)/bin/rusticle-tk
	rm -f $(LOCAL_PREFIX)/bin/rusticle-lsp
	rm -f $(LOCAL_PREFIX)/bin/kairn
	rm -rf $(LOCAL_PREFIX)/share/rusticle
	rm -rf $(LOCAL_PREFIX)/share/rusticle-tk
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
run-tk:
	@test -n "$(SCRIPT)" || (echo "Usage: make run-tk SCRIPT=path.tcl" && exit 1)
	cargo run -p rusticle-tk -- $(SCRIPT)

# Run rusticle-tk hello demo
demo-tk:
	cargo run -p rusticle-tk -- rusticle-tk/examples/hello.tcl
