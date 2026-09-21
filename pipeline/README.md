# pipeline

A Cargo workspace of three small CLI tools that implement a
[medallion pipeline](https://www.databricks.com/glossary/medallion-architecture)
over CSV data: `bronze` lands raw data with zero loss, `silver` cleans and
transforms it, and `gold` builds and verifies the final, production-ready
dataset. Each layer is generalized from the same patterns
([schema inference](common/src/schema.rs), [drift detection](common/src/drift.rs),
[declarative transforms](silver/src/transform.rs), [Parquet verification](common/src/check.rs))
so the course can teach one set of ideas and apply them at every stage.

## Workspace layout

- **`common`** — shared library: CSV summary statistics, schema
  inference/compliance, drift-profile building/comparison, a
  `DataQualityReport` type printed by every layer, and Parquet-check
  helpers.
- **`bronze`** — `bronze ingest` (CSV -> Parquet, zero data loss),
  `bronze schema infer|check`.
- **`silver`** — `silver clean` (drop empty columns, dedupe rows),
  `silver transform apply|scaffold` (declarative YAML pipeline).
- **`gold`** — `gold build` (feature engineering + completeness filter),
  `gold check` (Parquet integrity), `gold drift baseline|check`.

## Build & test

```bash
cargo build  --manifest-path pipeline/Cargo.toml
cargo test   --manifest-path pipeline/Cargo.toml
cargo fmt    --manifest-path pipeline/Cargo.toml -- --check
cargo clippy --manifest-path pipeline/Cargo.toml --all-targets -- -D warnings
```

Or from the repo root: `make rust`.

## Worked example

Run from the repository root, against the bundled sample dataset
(`data/raw/orders.csv`, 620 rows of synthetic e-commerce orders with
injected missing values, duplicate rows, mixed date formats, inconsistent
region casing, and a fully-empty `notes` column — see
[`scripts/generate_sample_data.py`](../scripts/generate_sample_data.py)).

### 1. Bronze: ingest the raw CSV

```console
$ cargo run -q --manifest-path pipeline/Cargo.toml --bin bronze -- \
    ingest data/raw/orders.csv --output data/bronze/orders.parquet
bronze report for data/raw/orders.csv
  output file:        data/bronze/orders.parquet
  rows in / out:       620 / 620 (0 dropped)
  nulls remaining:     668
```

Every row survives — bronze's only job is to land the data as typed,
columnar Parquet.

### 2. Silver: clean it

```console
$ cargo run -q --manifest-path pipeline/Cargo.toml --bin silver -- \
    clean data/bronze/orders.parquet --output data/silver/orders.parquet
silver clean report for data/bronze/orders.parquet
  output file:        data/silver/orders.parquet
  rows in / out:       620 / 600 (20 dropped)
  duplicates dropped:  20
  columns dropped:     1 (notes)
  nulls remaining:     45
```

The 20 exact-duplicate rows injected by the generator are gone, and the
fully-empty `notes` column is dropped.

### 3. Silver: apply a declarative transform

```console
$ cargo run -q --manifest-path pipeline/Cargo.toml --bin silver -- \
    transform scaffold --output data/silver/transform-spec.yaml
Wrote transform spec scaffold to data/silver/transform-spec.yaml

$ cargo run -q --manifest-path pipeline/Cargo.toml --bin silver -- \
    transform apply data/silver/orders.parquet \
    --spec data/silver/transform-spec.yaml \
    --output data/silver/orders.transformed.parquet
Transform summary for data/silver/orders.parquet
  output file:    data/silver/orders.transformed.parquet
  steps applied:  2
  rows in:        600
  rows out:       590 (98.3%)
```

The scaffolded spec normalizes the `region` column (trim + lowercase) and
drops rows with a missing `order_date`. Write your own spec with any
combination of `drop_columns`, `select_columns`, `rename_column`, `cast`,
`filter`, `normalize`, and `drop_nulls` steps — see
[`silver transform scaffold`](src) output for the full list, or
[`silver/src/transform.rs`](silver/src/transform.rs).

### 4. Gold: build and verify

```console
$ cargo run -q --manifest-path pipeline/Cargo.toml --bin gold -- \
    build data/silver/orders.transformed.parquet --output data/gold/orders.parquet
gold build report for data/silver/orders.transformed.parquet
  output file:        data/gold/orders.parquet
  rows in / out:       590 / 573 (17 dropped)
  nulls remaining:     18

$ cargo run -q --manifest-path pipeline/Cargo.toml --bin gold -- \
    check data/gold/orders.parquet
Parquet check for data/gold/orders.parquet
  status: VALID
  size: 13497 bytes
  rows: 573
  columns: 9
    order_id              i64         (0 nulls, 0.0%)
    customer_email        str         (0 nulls, 0.0%)
    region                str         (11 nulls, 1.9%)
    category              str         (7 nulls, 1.2%)
    unit_price            f64         (0 nulls, 0.0%)
    quantity              i64         (0 nulls, 0.0%)
    order_date            str         (0 nulls, 0.0%)
    status                str         (0 nulls, 0.0%)
    revenue               f64         (0 nulls, 0.0%)
```

`gold build` computed the `revenue = unit_price * quantity` feature and
dropped any row still missing `unit_price` or `quantity`; `gold check`
confirms the Parquet file's magic bytes, decodes every row group, and
reports 0% nulls in every required column.

### 5. Schema and drift

```console
$ cargo run -q --manifest-path pipeline/Cargo.toml --bin bronze -- \
    schema infer data/raw/orders.csv --output data/raw/orders.schema.json
Inferred schema for data/raw/orders.csv
  columns: 9
  saved to: data/raw/orders.schema.json
  order_id                  integer  not null
  customer_email            text     not null
  region                    text     nullable
  category                  text     nullable
  unit_price                float    nullable
  quantity                  integer  nullable
  order_date                text     nullable
  status                    text     not null
  notes                     text     nullable

$ cargo run -q --manifest-path pipeline/Cargo.toml --bin gold -- \
    drift baseline data/raw/orders.csv --output data/raw/orders.drift.json
Drift baseline for data/raw/orders.csv
  rows: 620
  saved to: data/raw/orders.drift.json

$ cargo run -q --manifest-path pipeline/Cargo.toml --bin gold -- \
    drift check data/raw/orders.csv --baseline data/raw/orders.drift.json
Drift check for data/raw/orders.csv
  against: data/raw/orders.drift.json
  status: NO DRIFT
```

`bronze schema check <file> --schema <schema.json>` exits `1` on a schema
violation (missing/extra columns, type mismatches, or a column that gained
nulls it wasn't allowed to have); `gold drift check <file> --baseline
<drift.json>` exits `1` on drift by default (`--mode warn` to only print
findings). Both are suitable as CI gates on incoming raw batches.

Run the whole sequence in one shot with `make pipeline-demo` from the repo
root.
