use std::fs;

use common::io::OutputFormat;

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("gold-test-{name}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn build_adds_revenue_and_drops_incomplete_rows() {
    let dir = temp_dir("build");
    let input = dir.join("in.csv");
    fs::write(&input, "unit_price,quantity\n10.0,2\n5.0,\n").unwrap();
    let output = dir.join("out.parquet");

    gold::build::run(&input, Some(output.clone()), OutputFormat::Parquet).unwrap();

    let report = common::check::verify_parquet(&output).unwrap();
    assert!(report.is_valid());
    assert_eq!(report.rows, 1, "row missing quantity should be dropped");
    assert!(report.columns.iter().any(|c| c.name == "revenue"));

    fs::remove_dir_all(&dir).ok();
}
