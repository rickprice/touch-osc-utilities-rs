//! Validation should produce no false-positive errors against any of
//! the real fixtures (they loaded fine in the actual editor).

use touch_osc_core::{validate, Layout};

fn fixtures_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures")
}

#[test]
fn no_fixture_reports_a_validation_error() {
    let dir = fixtures_dir();
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("tosc"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty());

    for path in paths {
        let layout = Layout::from_file(&path).unwrap();
        let issues = validate::validate(&layout);
        let errors: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == validate::Severity::Error)
            .collect();
        assert!(
            errors.is_empty(),
            "{path:?} unexpectedly failed validation: {errors:#?}"
        );
    }
}
