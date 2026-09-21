use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;

use common::drift::{self, DriftMode};
use common::summary::summarize_csv;

#[derive(Subcommand, Debug)]
pub enum DriftAction {
    /// Capture a drift baseline from a CSV batch: numeric min/max/mean ranges
    /// plus categorical value fingerprints for text columns.
    Baseline {
        file: PathBuf,

        #[arg(short, long, default_value = ",")]
        delimiter: char,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(long, default_value_t = 1000)]
        max_categories: usize,
    },
    /// Check a new CSV batch against a previously saved drift baseline.
    Check {
        file: PathBuf,

        #[arg(short, long)]
        baseline: PathBuf,

        #[arg(short, long, default_value = ",")]
        delimiter: char,

        #[arg(long, value_enum, default_value_t = DriftMode::Fail)]
        mode: DriftMode,

        #[arg(long, default_value_t = 0.0)]
        tolerance: f64,

        #[arg(long, default_value_t = 20.0)]
        category_tolerance: f64,
    },
}

fn print_baseline(file: &str, profile: &drift::DriftProfile, saved_to: &std::path::Path) {
    println!("Drift baseline for {file}");
    println!("  rows: {}", profile.rows);
    println!("  saved to: {}", saved_to.display());
}

fn print_report(report: &drift::DriftReport, baseline_path: &std::path::Path) {
    println!("Drift check for {}", report.file);
    println!("  against: {}", baseline_path.display());
    let label = if !report.has_drift() {
        "NO DRIFT"
    } else if report.mode == DriftMode::Warn {
        "WARNING: DRIFT DETECTED"
    } else {
        "DRIFT DETECTED"
    };
    println!("  status: {label}");
    for d in &report.numeric_drifts {
        println!(
            "    {}: observed {} outside baseline range {}..{}",
            d.column, d.observed, d.baseline_min, d.baseline_max
        );
    }
    for d in &report.categorical_drifts {
        println!(
            "    {}: baseline {} categories, observed {} ({:.1}% change)",
            d.column, d.baseline_distinct, d.observed_distinct, d.pct_change
        );
    }
}

pub fn run(action: DriftAction) -> Result<bool> {
    match action {
        DriftAction::Baseline {
            file,
            delimiter,
            output,
            max_categories,
        } => {
            let delim_byte = common::delimiter_byte(delimiter)?;
            let summary = summarize_csv(&file, delim_byte, true)?;
            let profile = drift::build_profile(&summary, max_categories);
            let output_path = output.unwrap_or_else(|| drift::default_drift_path(&file));
            drift::save_profile(&profile, &output_path)?;
            print_baseline(&file.display().to_string(), &profile, &output_path);
            Ok(true)
        }
        DriftAction::Check {
            file,
            baseline,
            delimiter,
            mode,
            tolerance,
            category_tolerance,
        } => {
            let delim_byte = common::delimiter_byte(delimiter)?;
            let baseline_profile = drift::load_profile(&baseline)?;
            let summary = summarize_csv(&file, delim_byte, true)?;
            let actual = drift::build_profile(&summary, usize::MAX);
            let report = drift::check_drift(
                &baseline_profile,
                &actual,
                &file.display().to_string(),
                mode,
                tolerance,
                category_tolerance,
            );
            let should_fail = report.should_fail();
            print_report(&report, &baseline);
            Ok(!should_fail)
        }
    }
}
