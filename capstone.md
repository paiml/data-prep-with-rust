# Capstone: Data Prep for ML

## Project Overview

Take a raw, messy dataset and turn it into a documented, versioned,
production-ready artifact by running it through your own three-tool
medallion pipeline: `bronze` (land it, lose nothing), `silver` (clean it,
deduplicate it), and `gold` (feature-engineer it, verify it). The project
exercises everything from the course: schema inference and compliance
checking, schema-drift detection between batches, declarative Polars
transforms, and Parquet packaging with an integrity check.

The deliverable is the three-crate Rust workspace in [`pipeline/`](pipeline/)
plus a written data-quality report comparing your pipeline's runtime and
memory footprint to a Python/pandas baseline on the same dataset. Every
crate compiles cleanly under `cargo clippy --all-targets -- -D warnings` and
is gated by `make check`.

You can use `data/raw/orders.csv` (the bundled sample dataset — see
[`scripts/generate_sample_data.py`](scripts/generate_sample_data.py)) or
bring your own raw CSV. Either way, the deliverables below apply.

## Deliverables

### 1. Bronze: land the raw data

Run `bronze ingest` against your raw CSV and confirm the output Parquet file
has exactly as many rows as the input — bronze's guarantee is that nothing
is lost, only the representation changes (typed, columnar Parquet instead of
untyped text). Also run `bronze schema infer` to capture a schema baseline,
and `bronze schema check` against a second batch (or a deliberately modified
copy) to show a schema violation being caught.

### 2. Silver: clean it

Run `silver clean` against the bronze output and record, from its printed
report: rows in/out, duplicate rows dropped, and any fully-empty columns
dropped. Then write your own `silver transform` YAML spec (start from
`silver transform scaffold`) that does at least three of: `drop_columns`,
`cast`, `filter`, `select_columns`, `rename_column`, `normalize`,
`drop_nulls` — and apply it with `silver transform apply`.

### 3. Gold: package it

Run `gold build` against your transformed silver output. Verify the result
with `gold check` and confirm the printed report shows `status: VALID`, the
expected row count, and 0% nulls in every required business column. Capture
a `gold drift baseline` from your original raw batch and a `gold drift
check` against a second, deliberately-shifted batch (or the same batch with
a tolerance of `0`) to demonstrate drift detection actually firing.

### 4. Data-quality report

Write a short (half-page to one-page) report, in Markdown, that includes:

- Before/after row and column counts at every stage of the pipeline (bronze
  -> silver -> gold), pulled from each tool's printed `DataQualityReport`.
- The specific data-quality issues your pipeline handled (missing values,
  duplicate rows, empty columns, mixed casing, schema drift, etc.) and how
  each was resolved.
- A benchmark: wall-clock time and peak memory (`/usr/bin/time -v` or
  equivalent) for your Rust pipeline versus an equivalent pandas script
  performing the same cleaning steps, on the same input file. Numbers, not
  vibes — include the exact commands you ran.

### 5. Test suite

The workspace ships tests that exercise the pipeline's guarantees:

- `common`: schema inference and compliance checking, drift-profile
  building and comparison, Parquet-check validity rules.
- `bronze`: ingest preserves every row.
- `silver`: clean drops fully-empty columns and deduplicates exact-match
  rows.
- `gold`: build computes the `revenue` feature and drops incomplete rows.

The full suite passes with `cargo test --manifest-path pipeline/Cargo.toml`,
and the build is clean under `cargo clippy --all-targets -- -D warnings`.
`make check` (lint + rust + validate) passes.

## Evaluation Criteria

### Advanced

- Full bronze -> silver -> gold run demonstrated end to end against a real
  dataset, with every stage's `DataQualityReport` captured in the written
  report.
- A custom `silver transform` YAML spec (not just the bundled scaffold
  example) applying at least three distinct step types, with rows in/out
  verified.
- Schema-drift detection demonstrated actually firing — a numeric range or
  categorical cardinality shift caught by `gold drift check`, not just a
  clean "no drift" run.
- A real Python/pandas benchmark with recorded wall-clock and memory
  numbers, not an estimate.
- `gold check` reports `status: VALID` with 0% nulls in every required
  business column; `cargo test` and `clippy -D warnings` are both clean.

### Proficient

- A working bronze -> silver -> gold run with all three
  `DataQualityReport`s recorded.
- Schema inference and a schema-compliance check both demonstrated (a
  compliant and a non-compliant run).
- `cargo test` passes; `clippy` is clean; no `unwrap`/`expect` on a data-read
  path.
- Benchmark against Python is present but rough (an estimate rather than a
  measured run).

### Developing

- Bronze and silver stages run, but gold is missing or its check reports
  `INVALID`.
- No custom transform spec — only the bundled scaffold example was applied
  unmodified.
- No drift detection demonstrated.
- Tests cover the happy path only; clippy warnings may be present.

## Resources

- **Polars user guide**: <https://docs.pola.rs/>
- **Polars Rust API docs**: <https://docs.rs/polars/latest/polars/>
- **Apache Parquet format**: <https://parquet.apache.org/docs/>
- **Apache Arrow (in-memory columnar format)**: <https://arrow.apache.org/overview/>
- **Medallion architecture** (Databricks' original write-up of the
  bronze/silver/gold pattern): <https://www.databricks.com/glossary/medallion-architecture>
- **`clap` derive API** (the CLI framework used by all three tools):
  <https://docs.rs/clap/latest/clap/_derive/index.html>
- **`anyhow`** (error handling used throughout): <https://docs.rs/anyhow/latest/anyhow/>

Reference implementation:

- **The pipeline workspace**: `pipeline/` in this repository —
  `pipeline/common/src/` for the shared schema/drift/quality types,
  `pipeline/{bronze,silver,gold}/src/` for each layer's CLI.
- **Pipeline usage walkthrough**: [`pipeline/README.md`](pipeline/README.md)
  — every command in this capstone, runnable against the bundled sample
  dataset.

## Share Your Work

Add this capstone to your LinkedIn profile as a portfolio project:

1. On your LinkedIn profile, choose **Add profile section** -> **Projects**.
2. **Title:** *Data Prep for ML — built a Rust + Polars medallion pipeline,
   directed with an AI coding agent*.
3. **Description:** Summarize the deliverables — the bronze/silver/gold CLI
   pipeline, the schema and drift checks, the declarative transform spec,
   and the benchmark against a Python/pandas baseline. Note that every stage
   produces an auditable data-quality report.
4. **Link:** the URL to your public GitHub repository for the project.
