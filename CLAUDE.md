# CLAUDE.md

Guidance for Claude Code (and any contributor) when working in this repository.

This repo is a Cargo workspace (`pipeline/common`, `pipeline/bronze`,
`pipeline/silver`, `pipeline/gold`) plus course materials (`outline.md`,
`capstone.md`, `coursera-assets/`). Code changes should follow the same
workflow the course itself teaches: state your intent, let the agent act,
then review the diff and check it against the actual data — never accept a
plausible-looking result without running it against `data/raw/orders.csv`.

## Core workflow: Test-Driven Development

This project follows strict TDD. For every fix or improvement:

1. **Write a failing test first.** Before changing any implementation code,
   add or modify a test in `tests/` (or a `#[cfg(test)]` module) that
   captures the bug or the new behavior. Run `cargo test` and confirm it
   fails for the expected reason.
2. **Make the minimal change** to `src/` needed to make that test pass.
3. **Run `cargo test` again** and confirm the full suite passes.

Never write implementation code before there is a failing test that
justifies it.

### Never add a feature without a test

Every new feature, flag, or code path must ship with a test that exercises
it. If a change has no test covering it, it is not done. Bug fixes need a
regression test that fails without the fix and passes with it.

## Required checks before considering any change complete

```sh
cargo fmt --manifest-path pipeline/Cargo.toml                                        # format the code
cargo clippy --manifest-path pipeline/Cargo.toml --all-targets -- -D warnings        # lint, warnings denied
cargo build --manifest-path pipeline/Cargo.toml                                      # confirm it compiles
cargo test --manifest-path pipeline/Cargo.toml                                       # confirm the full suite passes
```

Or simply use the Makefile target that bundles these:

```sh
make check   # lint + rust (fmt-check + clippy + test) + validate
```

If `make check` reports formatting issues, run `make fmt` to fix them, then
re-run `make check`.

## Repository shape

- `pipeline/common` — shared schema inference, summary statistics, drift
  detection, quality-report, and Parquet-check types used by all three
  binaries. Changes here ripple through bronze, silver, and gold — add tests
  in `common` itself before touching the layer crates.
- `pipeline/bronze` — `bronze ingest` (CSV -> Parquet, zero data loss) and
  `bronze schema infer|check`.
- `pipeline/silver` — `silver clean` (drop empty columns, dedupe rows) and
  `silver transform apply|scaffold` (declarative YAML pipeline).
- `pipeline/gold` — `gold build` (feature engineering + completeness
  filtering), `gold check` (Parquet integrity), `gold drift baseline|check`.

## Summary of rules

- TDD only: failing test -> implementation -> passing test. No exceptions.
- No feature or fix lands without a test that would fail without it.
- Always run `cargo fmt` and `cargo clippy -- -D warnings` before finishing.
- Always confirm `cargo build` (and `cargo test`) succeed before finishing.
- Prefer `make check` as the single pre-commit gate.
- Keep `data/raw/orders.csv` as the single source of truth for the demo
  pipeline; regenerate it with `scripts/generate_sample_data.py` rather than
  hand-editing rows.
