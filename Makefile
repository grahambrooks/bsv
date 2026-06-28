# Makefile for bsv repository administration.
#
# Run `make` (or `make help`) to list the available targets.
#
# Cutting a release:
#   make release VERSION=2026.3.0
# This bumps Cargo.toml, runs all checks, commits, tags vX.Y.Z and pushes.
# The Release workflow then builds the per-platform artifacts, publishes the
# GitHub Release, and regenerates the in-repo install packaging (Homebrew
# formula + Scoop manifest) with the new version and checksums.

SHELL := bash
.DEFAULT_GOAL := help

CARGO ?= cargo
CURRENT_VERSION := $(shell grep -m1 '^version = ' Cargo.toml | sed -E 's/.*"(.*)".*/\1/')

# `release`/`update-packaging` arguments (must be supplied on the command line).
VERSION ?=
# Directory of "<asset>.sha256" files for `update-packaging` (the Release
# workflow passes the downloaded artifacts; rarely needed by hand).
CHECKSUMS ?=

##@ General

.PHONY: help
help: ## Show this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nbsv — current version: $(CURRENT_VERSION)\n\nUsage: make \033[36m<target>\033[0m\n"} \
		/^[a-zA-Z0-9_-]+:.*##/ { printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2 } \
		/^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) }' $(MAKEFILE_LIST)

##@ Development

.PHONY: build
build: ## Build the debug binary
	$(CARGO) build

.PHONY: build-release
build-release: ## Build the optimized release binary
	$(CARGO) build --release

.PHONY: run
run: ## Run bsv against the current directory
	$(CARGO) run

.PHONY: install
install: ## Install bsv into the cargo bin directory
	$(CARGO) install --path .

.PHONY: doc
doc: ## Build and open the API docs
	$(CARGO) doc --no-deps --open

.PHONY: clean
clean: ## Remove build artifacts
	$(CARGO) clean

##@ Quality

.PHONY: fmt
fmt: ## Format the code
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check: ## Check formatting without modifying files
	$(CARGO) fmt --all -- --check

.PHONY: lint
lint: ## Run clippy with warnings denied
	$(CARGO) clippy --all-targets -- -D warnings

.PHONY: test
test: ## Run the full test suite (unit, integration, doctests)
	$(CARGO) test

.PHONY: shellcheck
shellcheck: ## Lint the shell scripts (skipped if shellcheck is absent)
	@if command -v shellcheck >/dev/null 2>&1; then \
		shellcheck install.sh scripts/*.sh && echo "shellcheck OK"; \
	else \
		echo "shellcheck not installed; skipping"; \
	fi

.PHONY: verify-packaging
verify-packaging: ## Smoke-test the Homebrew/Scoop manifest generator
	@set -e; \
	tmp=$$(mktemp -d); \
	trap 'cp "$$tmp/formula.orig" Formula/bsv.rb 2>/dev/null || true; \
	      cp "$$tmp/bucket.orig" bucket/bsv.json 2>/dev/null || true; rm -rf "$$tmp"' EXIT; \
	cp Formula/bsv.rb "$$tmp/formula.orig"; \
	cp bucket/bsv.json "$$tmp/bucket.orig"; \
	for t in aarch64-apple-darwin x86_64-apple-darwin x86_64-unknown-linux-gnu; do \
		printf '%s  bsv-%s.tar.gz\n' "$$(printf '%064d' 0)" "$$t" > "$$tmp/bsv-$$t.tar.gz.sha256"; \
	done; \
	printf '%s  bsv-x86_64-pc-windows-msvc.zip\n' "$$(printf '%064d' 0)" \
		> "$$tmp/bsv-x86_64-pc-windows-msvc.zip.sha256"; \
	scripts/update-packaging.sh 9.9.9 "$$tmp" >/dev/null; \
	grep -q 'version "9.9.9"' Formula/bsv.rb; \
	python3 -m json.tool bucket/bsv.json >/dev/null; \
	echo "packaging generator OK"

.PHONY: check
check: fmt-check lint test shellcheck verify-packaging ## Run every CI check
	@echo "All checks passed."

.PHONY: ci
ci: check ## Alias for check

##@ Release

.PHONY: version
version: ## Print the current crate version
	@echo "$(CURRENT_VERSION)"

.PHONY: update-packaging
update-packaging: ## Regenerate manifests: make update-packaging VERSION=x.y.z CHECKSUMS=dir
	@test -n "$(VERSION)"   || { echo "error: VERSION is required" >&2; exit 1; }; \
	test -n "$(CHECKSUMS)" || { echo "error: CHECKSUMS (dir of *.sha256) is required" >&2; exit 1; }; \
	scripts/update-packaging.sh "$(VERSION)" "$(CHECKSUMS)"

.PHONY: release
release: ## Cut a release: make release VERSION=2026.3.0
	@test -n "$(VERSION)" || { echo "error: VERSION is required, e.g. make release VERSION=2026.3.0" >&2; exit 1; }
	@scripts/release.sh "$(VERSION)"
