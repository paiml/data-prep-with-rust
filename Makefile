SHELL := /bin/bash
.DEFAULT_GOAL := help

PIPELINE := pipeline/Cargo.toml

.PHONY: help lint lint-md rust fmt validate check pipeline-demo clean

help: ## Show this help
	grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
	awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-16s\033[0m %s\n", $$1, $$2}'

lint: lint-md ## Markdown lint (no tabs)
	@echo "=== Markdown lint passed ==="

lint-md: ## Reject literal tabs in markdown
	@echo "=== Markdown lint (no tabs) ==="
	@find . -name '*.md' -not -path './.git/*' -not -path '*/target/*' | \
		xargs -I{} sh -c 'grep -Pn "\t" "$$1" && echo "FAIL: tab in $$1" && exit 1 || true' _ {}
	@echo "  no tabs found"

fmt: ## Format the Rust workspace
	cd pipeline && cargo fmt

rust: ## fmt-check + clippy + test the Rust workspace
	cd pipeline && cargo fmt -- --check
	cargo clippy --manifest-path $(PIPELINE) --all-targets -- -D warnings
	cargo test --manifest-path $(PIPELINE)

validate: ## Structural checks (outline, capstone, coursera-assets, README)
	@echo "=== Course structure ==="
	@test -f outline.md || { echo "FAIL: outline.md missing"; exit 1; }
	@test -f capstone.md || { echo "FAIL: capstone.md missing"; exit 1; }
	@for s in "## Deliverables" "## Evaluation Criteria" "## Share Your Work"; do \
		grep -qF "$$s" capstone.md || { echo "FAIL: capstone.md missing $$s"; exit 1; }; done
	@test $$(ls coursera-assets/*.md 2>/dev/null | wc -l) -ge 1 || { echo "FAIL: no coursera-assets"; exit 1; }
	@grep -q "Data Prep for ML" README.md || { echo "FAIL: README missing course title"; exit 1; }
	@test -f data/raw/orders.csv || { echo "FAIL: sample dataset missing"; exit 1; }
	@echo "  outline + capstone (3 sections) + coursera-assets + sample data + README present"

check: lint rust validate ## Full gate
	@echo "=== All quality gates passed ==="

pipeline-demo: ## Run bronze -> silver -> gold against the sample dataset
	cargo run --manifest-path $(PIPELINE) --bin bronze -- ingest data/raw/orders.csv --output data/bronze/orders.parquet
	cargo run --manifest-path $(PIPELINE) --bin silver -- clean data/bronze/orders.parquet --output data/silver/orders.parquet
	cargo run --manifest-path $(PIPELINE) --bin gold -- build data/silver/orders.parquet --output data/gold/orders.parquet
	cargo run --manifest-path $(PIPELINE) --bin gold -- check data/gold/orders.parquet

clean: ## Remove build artifacts
	cargo clean --manifest-path $(PIPELINE)
