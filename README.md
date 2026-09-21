# Data Prep for ML

**From Messy Data to Production-Ready Datasets using agentic tooling**

Learn to clean, validate, and transform data for machine learning using
[Polars](https://pola.rs) — a fast, memory-efficient DataFrame library —
directed through an AI coding agent in Rust. You'll take a raw, messy
dataset and turn it into a documented, versioned, production-ready
artifact, benchmarking your pipeline's speed and memory footprint against a
Python baseline along the way. Designed for learners new to Rust; no prior
Rust experience required.

## Table of Contents

- [What You'll Learn](#what-youll-learn)
- [Course Outline](#course-outline)
- [Hands-On: the medallion pipeline](#hands-on-the-medallion-pipeline)
- [Repository Layout](#repository-layout)
- [Capstone](#capstone)
- [Instructor](#instructor)
- [License](#license)

## What You'll Learn

The course climbs from "why Rust for data prep at all" to a working,
tested, three-stage pipeline, directed the whole way by an AI coding agent.
By the end you can:

- **Explain why Rust pairs well with agentic coding** — a strict type
  system and "nothing runs until it compiles" turn the compiler into a
  tight, precise correction loop for a large language model, and you stay
  the reviewer who decides which diff lands.
- **Infer and enforce a schema** — a schema is a promise about your data
  (name + type + nullability per column); catch a violation automatically
  instead of discovering it downstream.
- **Detect schema drift between data batches** — beyond shape, catch a
  numeric range shifting or a categorical column's cardinality exploding,
  with configurable tolerance.
- **Write declarative Polars transforms** — a YAML spec of steps
  (`drop_columns`, `cast`, `filter`, `select_columns`, `rename_column`,
  `normalize`, `drop_nulls`) compiled into a single optimized lazy query
  plan, instead of hand-written imperative cleaning code.
- **Build a medallion pipeline** — three small, focused CLI tools (bronze,
  silver, gold), each with one guarantee, composing through the filesystem
  into a full raw-to-production pipeline.
- **Package and verify a production dataset** — write Parquet, and verify
  it byte-for-byte (magic bytes, decode integrity, null percentages) before
  calling it done.
- **Benchmark against a Python baseline** — put a number on what Rust +
  Polars actually buys you over pandas, on your own dataset.

## Course Outline

Four weeks, ten lessons, one capstone. The full lesson-by-lesson breakdown,
including short descriptions for every lesson, lives in
[`outline.md`](outline.md).

| Week | Lesson | Title |
|---|---|---|
| 1 · Orient | 1.1 | Why Data Prep in Rust |
| 1 · Orient | 1.2 | Introduction to Agentic Coding |
| 1 · Orient | 1.3 | Challenges with Agentic Coding |
| 2 · Build | 2.1 | Creating your first tool |
| 2 · Build | 2.2 | Enhancing a data tool |
| 2 · Build | 2.3 | Handling Real-World Messy Data |
| 3 · Medallion Pipeline | 3.1 | The medallion architecture |
| 3 · Medallion Pipeline | 3.2 | The CLI pipeline |
| 3 · Medallion Pipeline | 3.3 | Packaging the Cleaned Dataset |
| 4 · Finish | 4.1 | Course Wrap-Up |

Week 3 is the hands-on core: three CLI tools that implement the Bronze,
Silver, and Gold layers of a medallion pipeline, built and tested with an AI
coding agent (Claude Code) directing the work.

## Hands-On: the medallion pipeline

The capstone artifact is [`pipeline/`](pipeline/) — a Cargo workspace with
three binaries that compose a raw-to-production data pipeline:

```text
data/raw/orders.csv (messy CSV)
        │  bronze ingest
        ▼
data/bronze/orders.parquet      — everything preserved, just typed & columnar
        │  silver clean, silver transform apply
        ▼
data/silver/orders.parquet      — deduplicated, empty columns dropped, transformed
        │  gold build
        ▼
data/gold/orders.parquet        — feature-engineered, complete, verified
```

Each layer has one guarantee (see [`outline.md`](outline.md) lesson 3.1.2):

- **Bronze** guarantees nothing is lost — the raw CSV is read as-is and
  landed as typed, columnar Parquet.
- **Silver** guarantees every remaining row is distinct and every remaining
  column carries information — empty columns and exact-duplicate rows are
  dropped, and a declarative YAML spec applies further transforms.
- **Gold** guarantees every row is complete and ready for direct use — a
  `revenue` feature is computed and any row still missing a required
  business field is dropped, then the output is verified with `gold check`.

Every stage prints a `DataQualityReport` (rows in/out, duplicates dropped,
columns dropped, nulls remaining) so a run can be audited without
re-reading the data. Build, lint, and test the whole workspace:

```bash
make rust            # fmt --check + clippy -D warnings + cargo test
make pipeline-demo    # bronze -> silver -> gold against the bundled sample data
make check            # full gate: lint + rust + validate
```

`make rust` is a thin wrapper over the standard toolchain, so you can also
work directly with `pipeline/`:

```bash
cargo build  --manifest-path pipeline/Cargo.toml
cargo test   --manifest-path pipeline/Cargo.toml
cargo clippy --manifest-path pipeline/Cargo.toml --all-targets -- -D warnings
```

The full command reference — schema inference, drift baselines, transform
specs, every flag — is in [`pipeline/README.md`](pipeline/README.md).

## Repository Layout

```text
data-prep-with-rust/
├── outline.md              # Week/lesson outline (source of the table above)
├── capstone.md              # Capstone brief: deliverables, rubric, share-your-work
├── README.md                # This file
├── CLAUDE.md                 # TDD workflow guidance for agentic contributors
├── coursera-assets/          # Key-terms and reflections, one pair per module
├── data/
│   ├── raw/orders.csv                 # Bundled messy sample dataset
│   ├── raw/orders.schema.json         # Committed schema baseline example
│   ├── raw/orders.drift.json          # Committed drift baseline example
│   ├── bronze/  silver/  gold/        # Generated pipeline outputs (gitignored)
├── scripts/
│   └── generate_sample_data.py       # Regenerates data/raw/orders.csv
├── pipeline/                # The Cargo workspace (the capstone artifact)
│   ├── Cargo.toml
│   ├── common/               # Shared schema, summary, drift, quality-report types
│   ├── bronze/                # `bronze ingest`, `bronze schema infer|check`
│   ├── silver/                # `silver clean`, `silver transform apply|scaffold`
│   ├── gold/                  # `gold build`, `gold check`, `gold drift baseline|check`
│   └── README.md              # Full command reference and worked example
├── Makefile                 # help / lint / rust / validate / check / pipeline-demo
└── LICENSE                  # MIT
```

## Capstone

The capstone is to **run a raw, messy dataset through your own three-tool
medallion pipeline** and produce a written data-quality report benchmarking
it against a Python/pandas baseline. Full deliverables, the evaluation
rubric (Developing / Proficient / Advanced), and share-your-work guidance
are in [`capstone.md`](capstone.md).

## Instructor

- **Alfredo Deza**

## License

Course content and code examples — including the `pipeline/` workspace —
are released under the [MIT License](LICENSE).
