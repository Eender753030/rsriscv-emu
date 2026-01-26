mod common;

use std::path::Path;
use std::fs;

#[test]
fn test_rv32mi_p() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let test_dir = Path::new(manifest_dir).join("tests/data/rv32mi-p");

    if !test_dir.exists() {
        eprintln!("Skipping rv32mi-p tests: Directory not found at {:?}", test_dir);
        return;
    }

    let mut paths: Vec<_> = fs::read_dir(test_dir)
        .unwrap()
        .map(|res| res.unwrap().path())
        .filter(|path| {
            path.is_file() && !path.file_name().unwrap().to_string_lossy().starts_with('.')
        })
        .collect();
    
    paths.sort();

    for path in paths {
        common::run_test_file(&path);
    }
}