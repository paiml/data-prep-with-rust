use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use anyhow::{Context, Result};
use polars::prelude::*;

/// Verification details for a single column in a Parquet file.
#[derive(Debug, Clone)]
pub struct ColumnCheck {
    pub name: String,
    pub dtype: String,
    pub null_count: usize,
    pub null_pct: f64,
}

/// Complete verification report for a Parquet file.
#[derive(Debug, Clone)]
pub struct ParquetCheckReport {
    pub file: String,
    pub file_size_bytes: u64,
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub rows: usize,
    pub columns: Vec<ColumnCheck>,
}

impl ParquetCheckReport {
    pub fn is_valid(&self) -> bool {
        self.valid && self.errors.is_empty()
    }
}

fn polars_result<T>(r: PolarsResult<T>) -> Result<T> {
    r.map_err(anyhow::Error::from)
}

/// Verify a Parquet file: magic bytes, decode integrity, schema, and null
/// percentages per column. Used by `gold check` to confirm a packaged
/// dataset is a valid, readable production artifact.
pub fn verify_parquet(path: &Path) -> Result<ParquetCheckReport> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let metadata = file
        .metadata()
        .with_context(|| format!("failed to read metadata of {}", path.display()))?;
    let file_size = metadata.len();

    let mut report = ParquetCheckReport {
        file: path.display().to_string(),
        file_size_bytes: file_size,
        valid: true,
        errors: Vec::new(),
        warnings: Vec::new(),
        rows: 0,
        columns: Vec::new(),
    };

    if file_size < 8 {
        report.valid = false;
        report.errors.push(format!(
            "file is too small to be a valid Parquet file (size: {file_size} bytes, minimum: 8 bytes)"
        ));
        return Ok(report);
    }

    let mut header = [0u8; 4];
    file.read_exact(&mut header)
        .with_context(|| format!("failed to read header of {}", path.display()))?;
    if &header != b"PAR1" {
        report.valid = false;
        report.errors.push(format!(
            "invalid magic header: expected 'PAR1', found '{}'",
            String::from_utf8_lossy(&header)
        ));
    }

    let mut footer = [0u8; 4];
    file.seek(SeekFrom::End(-4))
        .with_context(|| format!("failed to seek to footer of {}", path.display()))?;
    file.read_exact(&mut footer)
        .with_context(|| format!("failed to read footer of {}", path.display()))?;
    if &footer != b"PAR1" {
        report.valid = false;
        report.errors.push(format!(
            "invalid magic footer: expected 'PAR1', found '{}'",
            String::from_utf8_lossy(&footer)
        ));
    }

    if !report.valid {
        return Ok(report);
    }

    file.seek(SeekFrom::Start(0))
        .with_context(|| format!("failed to rewind {}", path.display()))?;

    let df = match polars_result(ParquetReader::new(file).finish()) {
        Ok(df) => df,
        Err(err) => {
            report.valid = false;
            report
                .errors
                .push(format!("failed to read parquet data: {err:#}"));
            return Ok(report);
        }
    };

    report.rows = df.height();
    let columns = df.columns();

    if columns.is_empty() {
        report.warnings.push("file has 0 columns".to_string());
    }
    if df.height() == 0 {
        report
            .warnings
            .push("file contains 0 rows (empty dataset)".to_string());
    }

    let mut seen_names = std::collections::HashSet::new();
    for (idx, col) in columns.iter().enumerate() {
        let name = col.name().to_string();
        if name.is_empty() {
            report
                .warnings
                .push(format!("column at index {idx} has an empty column name"));
        } else if !seen_names.insert(name.clone()) {
            report
                .warnings
                .push(format!("duplicate column name '{name}'"));
        }

        let dtype = format!("{}", col.dtype());
        let null_count = col.null_count();
        let null_pct = if df.height() > 0 {
            (null_count as f64 / df.height() as f64) * 100.0
        } else {
            0.0
        };

        if df.height() > 0 && null_count == df.height() {
            let label = if name.is_empty() {
                format!("column at index {idx} (unnamed)")
            } else {
                format!("column '{name}'")
            };
            report
                .warnings
                .push(format!("{label} is 100.0% null ({null_count} nulls)"));
        }

        report.columns.push(ColumnCheck {
            name,
            dtype,
            null_count,
            null_pct,
        });
    }

    Ok(report)
}

pub fn print_parquet_check(report: &ParquetCheckReport) {
    println!("Parquet check for {}", report.file);
    println!(
        "  status: {}",
        if report.is_valid() {
            "VALID"
        } else {
            "INVALID"
        }
    );
    println!("  size: {} bytes", report.file_size_bytes);

    if !report.errors.is_empty() {
        println!("  errors:");
        for err in &report.errors {
            println!("    - {err}");
        }
    }

    if report.valid {
        println!("  rows: {}", report.rows);
        println!("  columns: {}", report.columns.len());
        for col in &report.columns {
            println!(
                "    {:<20}  {:<10}  ({} nulls, {:.1}%)",
                col.name, col.dtype, col.null_count, col.null_pct
            );
        }
    }

    if !report.warnings.is_empty() {
        println!("  warnings:");
        for w in &report.warnings {
            println!("    - {w}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_small_file_is_invalid() {
        let dir = std::env::temp_dir().join(format!("common-check-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tiny.parquet");
        std::fs::write(&path, b"abc").unwrap();

        let report = verify_parquet(&path).unwrap();
        assert!(!report.is_valid());
        std::fs::remove_dir_all(&dir).ok();
    }
}
