use anyhow::Result;
use bronze::{ingest, schema_cmd};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use common::io::OutputFormat;

/// bronze: land raw CSV data as typed, columnar Parquet with zero data loss.
#[derive(Parser, Debug)]
#[command(name = "bronze", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Ingest a raw CSV file into the bronze landing zone as Parquet, preserving
    /// every row and column exactly as they appeared in the source.
    Ingest {
        file: PathBuf,

        #[arg(short, long, default_value = ",")]
        delimiter: char,

        #[arg(long)]
        no_header: bool,

        /// Output path. Defaults to "data/bronze/<name>.parquet".
        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(short = 'f', long = "output-format", value_enum, default_value_t = OutputFormat::Parquet)]
        output_format: OutputFormat,
    },
    /// Infer or check a CSV's column schema.
    Schema {
        #[command(subcommand)]
        action: schema_cmd::SchemaAction,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Ingest {
            file,
            delimiter,
            no_header,
            output,
            output_format,
        } => ingest::run(&file, delimiter, no_header, output, output_format),
        Command::Schema { action } => {
            let ok = schema_cmd::run(action)?;
            if !ok {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}
