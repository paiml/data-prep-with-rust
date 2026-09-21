use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use common::io::OutputFormat;
use silver::{clean, transform};

/// silver: clean bronze data and apply declarative transforms.
#[derive(Parser, Debug)]
#[command(name = "silver", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Drop 100%-empty columns and deduplicate exact-match rows from a bronze batch.
    Clean {
        file: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(short = 'f', long = "output-format", value_enum, default_value_t = OutputFormat::Parquet)]
        output_format: OutputFormat,
    },
    /// Apply a declarative transform pipeline loaded from a YAML spec.
    Transform {
        #[command(subcommand)]
        action: TransformAction,
    },
}

#[derive(Subcommand, Debug)]
enum TransformAction {
    /// Apply a YAML transform spec to a file.
    Apply {
        file: PathBuf,

        #[arg(short, long)]
        spec: PathBuf,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(short = 'f', long = "output-format", value_enum, default_value_t = OutputFormat::Parquet)]
        output_format: OutputFormat,
    },
    /// Write a YAML transform spec scaffold documenting every available step type.
    Scaffold {
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Clean {
            file,
            output,
            output_format,
        } => clean::run(&file, output, output_format),
        Command::Transform { action } => match action {
            TransformAction::Apply {
                file,
                spec,
                output,
                output_format,
            } => {
                let output_path =
                    output.unwrap_or_else(|| transform::default_output_path(&file, output_format));
                let spec = transform::load_spec(&spec)?;
                let report = transform::run_spec(&file, &spec, &output_path, output_format)?;
                report.print();
                Ok(())
            }
            TransformAction::Scaffold { output } => match output {
                Some(path) => {
                    transform::write_scaffold(&path)?;
                    println!("Wrote transform spec scaffold to {}", path.display());
                    Ok(())
                }
                None => {
                    print!("{}", transform::scaffold_yaml());
                    Ok(())
                }
            },
        },
    }
}
