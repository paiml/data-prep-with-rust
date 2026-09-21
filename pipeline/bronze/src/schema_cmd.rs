use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;

use common::schema;
use common::summary::summarize_csv;

#[derive(Subcommand, Debug)]
pub enum SchemaAction {
    /// Infer a schema from a CSV file and save it to disk.
    Infer {
        file: PathBuf,

        #[arg(short, long, default_value = ",")]
        delimiter: char,

        #[arg(long)]
        no_header: bool,

        /// Where to save the schema JSON. Defaults to "<name>.schema.json" next to the input file.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Check a CSV file against a previously saved schema.
    Check {
        file: PathBuf,

        #[arg(short, long)]
        schema: PathBuf,

        #[arg(short, long, default_value = ",")]
        delimiter: char,

        #[arg(long)]
        no_header: bool,
    },
}

fn print_schema(file: &str, s: &schema::Schema, saved_to: Option<&std::path::Path>) {
    println!("Inferred schema for {file}");
    println!("  columns: {}", s.columns.len());
    if let Some(path) = saved_to {
        println!("  saved to: {}", path.display());
    }
    for col in &s.columns {
        println!(
            "  {:<24}  {:<7}  {}",
            col.name,
            col.col_type.to_string(),
            if col.nullable { "nullable" } else { "not null" }
        );
    }
}

fn print_compliance(report: &schema::ComplianceReport, schema_path: &std::path::Path) {
    println!("Schema check for {}", report.file);
    println!("  against: {}", schema_path.display());
    println!(
        "  status: {}",
        if report.is_compliant() {
            "COMPLIANT"
        } else {
            "NOT COMPLIANT"
        }
    );
    if !report.missing_columns.is_empty() {
        println!("  missing columns: {}", report.missing_columns.join(", "));
    }
    if !report.extra_columns.is_empty() {
        println!("  extra columns: {}", report.extra_columns.join(", "));
    }
    if !report.type_mismatches.is_empty() {
        println!("  type mismatches:");
        for (name, expected, actual) in &report.type_mismatches {
            println!("    {name}: expected {expected}, found {actual}");
        }
    }
    if !report.nullability_violations.is_empty() {
        println!("  nullability violations:");
        for name in &report.nullability_violations {
            println!("    {name}: expected not null, found missing values");
        }
    }
}

pub fn run(action: SchemaAction) -> Result<bool> {
    match action {
        SchemaAction::Infer {
            file,
            delimiter,
            no_header,
            output,
        } => {
            let delim_byte = common::delimiter_byte(delimiter)?;
            let has_header = !no_header;
            let summary = summarize_csv(&file, delim_byte, has_header)?;
            let inferred = schema::infer_schema(&summary, delimiter, has_header);
            let output_path = output.unwrap_or_else(|| schema::default_schema_path(&file));
            schema::save_schema(&inferred, &output_path)?;
            print_schema(&file.display().to_string(), &inferred, Some(&output_path));
            Ok(true)
        }
        SchemaAction::Check {
            file,
            schema: schema_path,
            delimiter,
            no_header,
        } => {
            let delim_byte = common::delimiter_byte(delimiter)?;
            let has_header = !no_header;
            let expected = schema::load_schema(&schema_path)?;
            let summary = summarize_csv(&file, delim_byte, has_header)?;
            let actual = schema::infer_schema(&summary, delimiter, has_header);
            let report = schema::check_compliance(&expected, &actual, &file.display().to_string());
            let compliant = report.is_compliant();
            print_compliance(&report, &schema_path);
            Ok(compliant)
        }
    }
}
