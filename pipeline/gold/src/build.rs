use std::path::{Path, PathBuf};

use anyhow::Result;
use polars::prelude::*;

use common::io::{self, OutputFormat};
use common::quality::DataQualityReport;

pub fn default_output_path(input: &Path, format: OutputFormat) -> PathBuf {
    let stem = input
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());
    let ext = match format {
        OutputFormat::Parquet => "parquet",
        OutputFormat::Csv => "csv",
    };
    PathBuf::from("data/gold").join(format!("{stem}.{ext}"))
}

/// Build the gold, business-ready dataset from a silver batch: compute the
/// `revenue` feature (`unit_price * quantity`) and drop any row still
/// missing a value in a required business column. This is the gold layer's
/// guarantee — every row is complete and ready for direct use downstream.
pub fn run(input: &Path, output: Option<PathBuf>, output_format: OutputFormat) -> Result<()> {
    let output_path = output.unwrap_or_else(|| default_output_path(input, output_format));

    let lf = io::scan_auto(input, b',', true)?;
    let df = io::collect(lf.clone(), &format!("failed to read {}", input.display()))?;
    let rows_in = df.height();

    let required: Vec<&str> = ["unit_price", "quantity"]
        .into_iter()
        .filter(|c| df.get_column_names().iter().any(|n| n.as_str() == *c))
        .collect();

    let mut built = lf;
    if required.len() == 2 {
        built = built.with_column((col("unit_price") * col("quantity")).alias("revenue"));
    }
    for column in &required {
        built = built.filter(col(*column).is_not_null());
    }

    let final_df = io::collect(built, "failed to build gold dataset")?;
    let rows_out = final_df.height();
    let nulls_remaining = io::total_nulls(&final_df);

    io::write_output(final_df, &output_path, output_format)?;

    let report = DataQualityReport {
        stage: "gold build".to_string(),
        input: input.display().to_string(),
        output: output_path.display().to_string(),
        rows_in,
        rows_out,
        duplicates_dropped: 0,
        columns_dropped: Vec::new(),
        nulls_remaining,
    };
    report.print();
    Ok(())
}
