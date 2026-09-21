use std::path::Path;

use anyhow::{Context, Result};
use polars::prelude::*;

/// Target format for pipeline output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, Default)]
pub enum OutputFormat {
    Csv,
    #[default]
    Parquet,
}

impl OutputFormat {
    /// Attempt to infer the format from a file path's extension.
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "parquet" | "pq" => Some(OutputFormat::Parquet),
            "csv" => Some(OutputFormat::Csv),
            _ => None,
        }
    }
}

fn polars_result<T>(r: PolarsResult<T>) -> Result<T> {
    r.map_err(anyhow::Error::from)
}

/// Lazily scan a CSV file.
pub fn scan_csv(input: &Path, delim_byte: u8, has_header: bool) -> Result<LazyFrame> {
    let path = PlRefPath::new(input.to_string_lossy().as_ref());
    polars_result(
        LazyCsvReader::new(path)
            .with_separator(delim_byte)
            .with_has_header(has_header)
            .finish(),
    )
    .with_context(|| format!("failed to open {}", input.display()))
}

/// Lazily scan a Parquet file.
pub fn scan_parquet(input: &Path) -> Result<LazyFrame> {
    let path = PlRefPath::new(input.to_string_lossy().as_ref());
    polars_result(LazyFrame::scan_parquet(path, ScanArgsParquet::default()))
        .with_context(|| format!("failed to open {}", input.display()))
}

/// Scan either a CSV or a Parquet file, inferring the format from the extension.
pub fn scan_auto(input: &Path, delim_byte: u8, has_header: bool) -> Result<LazyFrame> {
    match OutputFormat::from_path(input) {
        Some(OutputFormat::Parquet) => scan_parquet(input),
        _ => scan_csv(input, delim_byte, has_header),
    }
}

/// Materialize a lazy frame and report its row count.
pub fn collect(lf: LazyFrame, context: &str) -> Result<DataFrame> {
    polars_result(lf.collect()).with_context(|| context.to_string())
}

/// Write a `DataFrame` out as CSV or Parquet.
pub fn write_output(mut df: DataFrame, output: &Path, format: OutputFormat) -> Result<()> {
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }
    let file = std::fs::File::create(output)
        .with_context(|| format!("failed to create {}", output.display()))?;
    match format {
        OutputFormat::Csv => {
            polars_result(CsvWriter::new(file).finish(&mut df))
                .with_context(|| format!("failed to write CSV to {}", output.display()))?;
        }
        OutputFormat::Parquet => {
            polars_result(ParquetWriter::new(file).finish(&mut df))
                .with_context(|| format!("failed to write Parquet to {}", output.display()))?;
        }
    }
    Ok(())
}

/// Count null values across every column of a `DataFrame`.
pub fn total_nulls(df: &DataFrame) -> usize {
    df.columns().iter().map(|c| c.null_count()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_format_inferred_from_extension() {
        assert_eq!(
            OutputFormat::from_path(Path::new("x.parquet")),
            Some(OutputFormat::Parquet)
        );
        assert_eq!(
            OutputFormat::from_path(Path::new("x.csv")),
            Some(OutputFormat::Csv)
        );
        assert_eq!(OutputFormat::from_path(Path::new("x.bin")), None);
    }
}
