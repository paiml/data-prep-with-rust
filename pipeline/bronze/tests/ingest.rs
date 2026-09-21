use std::fs;

use common::io::OutputFormat;

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("bronze-test-{name}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn ingest_preserves_every_row() {
    let dir = temp_dir("ingest-preserves");
    let input = dir.join("in.csv");
    fs::write(&input, "a,b\n1,x\n2,\n3,z\n").unwrap();
    let output = dir.join("out.parquet");

    bronze::ingest::run(
        &input,
        ',',
        false,
        Some(output.clone()),
        OutputFormat::Parquet,
    )
    .unwrap();

    let report = common::check::verify_parquet(&output).unwrap();
    assert!(report.is_valid());
    assert_eq!(report.rows, 3);

    fs::remove_dir_all(&dir).ok();
}
