use std::fs;

use common::io::OutputFormat;

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("silver-test-{name}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn clean_drops_empty_columns_and_dedupes() {
    let dir = temp_dir("clean");
    let input = dir.join("in.csv");
    fs::write(&input, "a,b,notes\n1,x,\n1,x,\n2,y,\n").unwrap();
    let output = dir.join("out.parquet");

    silver::clean::run(&input, Some(output.clone()), OutputFormat::Parquet).unwrap();

    let report = common::check::verify_parquet(&output).unwrap();
    assert!(report.is_valid());
    assert_eq!(report.rows, 2, "exact-duplicate row should be dropped");
    assert_eq!(
        report.columns.len(),
        2,
        "fully-empty `notes` column should be dropped"
    );

    fs::remove_dir_all(&dir).ok();
}
