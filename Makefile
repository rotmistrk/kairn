PREFIX ?= /usr/local
LOCAL_PREFIX ?= $(HOME)/.local
DEMO_DIR ?= $(LOCAL_PREFIX)/share/rusticle/examples
TK_DEMO_DIR ?= $(LOCAL_PREFIX)/share/rusticle-tk/examples

.PHONY: all check lint clippy test fmt clean build release \
        install-local uninstall-local \
        install-rusticle install-rusticle-tk install-kairn \
        install-demos verify

# ── Full cycle ──────────────────────────────────────────

all: fmt check lint test
	@echo "✅ All checks passed"

verify: fmt clippy test
	@echo "✅ Verification passed"

# ── Build ───────────────────────────────────────────────

check:
	cargo check --workspace

build:
	cargo build --workspace

release:
	cargo build --workspace --release

# ── Quality ─────────────────────────────────────────────

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace -- -D warnings

test:
	cargo test --workspace

lint: clippy

# ── Install (all to ~/.local/bin) ───────────────────────

install-local: install-rusticle install-rusticle-tk install-demos
	@echo "✅ Installed rusticle, rusticle-tk, and demos to $(LOCAL_PREFIX)"

install-rusticle: release
	install -d $(LOCAL_PREFIX)/bin
	install -m 755 target/release/rusticle $(LOCAL_PREFIX)/bin/rusticle
	@echo "  ✅ rusticle → $(LOCAL_PREFIX)/bin/rusticle"

install-rusticle-tk: release
	install -d $(LOCAL_PREFIX)/bin
	install -m 755 target/release/rusticle-tk $(LOCAL_PREFIX)/bin/rusticle-tk
	@echo "  ✅ rusticle-tk → $(LOCAL_PREFIX)/bin/rusticle-tk"

install-kairn: release
	install -d $(LOCAL_PREFIX)/bin
	install -m 755 target/release/kairn $(LOCAL_PREFIX)/bin/kairn
	@echo "  ✅ kairn → $(LOCAL_PREFIX)/bin/kairn"

install-demos:
	install -d $(DEMO_DIR)
	install -m 644 rusticle/examples/*.tcl $(DEMO_DIR)/
	@echo "  ✅ rusticle demos → $(DEMO_DIR)/"
	install -d $(TK_DEMO_DIR)
	install -m 644 rusticle-tk/examples/*.tcl $(TK_DEMO_DIR)/
	@echo "  ✅ rusticle-tk demos → $(TK_DEMO_DIR)/"

# ── Uninstall ───────────────────────────────────────────

uninstall-local:
	rm -f $(LOCAL_PREFIX)/bin/rusticle
	rm -f $(LOCAL_PREFIX)/bin/rusticle-tk
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

# Run all rusticle demos
demo-rusticle:
	@for f in rusticle/examples/*.tcl; do \
		echo "\n══════ $$f ══════"; \
		cargo run -p rusticle -- "$$f" || true; \
	done

# Run rusticle-tk hello demo
demo-tk:
	cargo run -p rusticle-tk -- rusticle-tk/examples/hello.tcl
