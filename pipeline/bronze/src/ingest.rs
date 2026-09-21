use std::path::{Path, PathBuf};

use anyhow::Result;

use common::io::{self, OutputFormat};
use common::quality::DataQualityReport;

/// Default bronze output path: `data/raw/orders.csv` -> `data/bronze/orders.parquet`.
pub fn default_output_path(input: &Path, format: OutputFormat) -> PathBuf {
    let stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());
    let ext = match format {
        OutputFormat::Parquet => "parquet",
        OutputFormat::Csv => "csv",
    };
    PathBuf::from("data/bronze").join(format!("{stem}.{ext}"))
}

/// Ingest a raw CSV into the bronze landing zone: read it as-is (no drops, no
/// casts, no filters) and write it back out as typed, columnar Parquet. The
/// bronze layer's guarantee is that nothing is lost — only the representation
/// changes.
pub fn run(
    file: &Path,
    delimiter: char,
    no_header: bool,
    output: Option<PathBuf>,
    output_format: OutputFormat,
) -> Result<()> {
    let delim_byte = common::delimiter_byte(delimiter)?;
    let has_header = !no_header;
    let output_path = output.unwrap_or_else(|| default_output_path(file, output_format));

    let lf = io::scan_csv(file, delim_byte, has_header)?;
    let df = io::collect(lf, &format!("failed to read {}", file.display()))?;
    let rows = df.height();
    let nulls = io::total_nulls(&df);

    io::write_output(df, &output_path, output_format)?;

    let report = DataQualityReport {
        stage: "bronze".to_string(),
        input: file.display().to_string(),
        output: output_path.display().to_string(),
        rows_in: rows,
        rows_out: rows,
        duplicates_dropped: 0,
        columns_dropped: Vec::new(),
        nulls_remaining: nulls,
    };
    report.print();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_output_lands_in_data_bronze() {
        let path = default_output_path(Path::new("data/raw/orders.csv"), OutputFormat::Parquet);
        assert_eq!(path, PathBuf::from("data/bronze/orders.parquet"));
    }
}
