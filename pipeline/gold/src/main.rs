use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use common::check;
use common::io::OutputFormat;
use gold::{build, drift_cmd};

/// gold: business-ready aggregates, packaging, integrity checks, and drift detection.
#[derive(Parser, Debug)]
#[command(name = "gold", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Build the final gold dataset from a silver batch.
    Build {
        file: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(short = 'f', long = "output-format", value_enum, default_value_t = OutputFormat::Parquet)]
        output_format: OutputFormat,
    },
    /// Verify the integrity of a packaged Parquet file.
    Check { file: PathBuf },
    /// Capture or check a data-drift baseline against incoming raw batches.
    Drift {
        #[command(subcommand)]
        action: drift_cmd::DriftAction,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Build {
            file,
            output,
            output_format,
        } => build::run(&file, output, output_format),
        Command::Check { file } => {
            let report = check::verify_parquet(&file)?;
            check::print_parquet_check(&report);
            if !report.is_valid() {
                std::process::exit(1);
            }
            Ok(())
        }
        Command::Drift { action } => {
            let ok = drift_cmd::run(action)?;
            if !ok {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}
