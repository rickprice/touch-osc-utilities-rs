//! End-to-end tests for the `tosc` binary itself (not just the library).
//! Everything else in this project is tested at the `touch-osc-core`
//! level; these confirm the CLI wiring -- argument parsing, exit codes,
//! stdout/file output, `--force` -- actually behaves as documented.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures")
}

fn tosc() -> Command {
    Command::cargo_bin("tosc").unwrap()
}

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("tosc-cli-test-{}-{name}", std::process::id()))
}

#[test]
fn dump_prints_yaml_to_stdout() {
    tosc()
        .arg("dump")
        .arg(fixtures_dir().join("blank.tosc"))
        .assert()
        .success()
        .stdout(predicate::str::contains("lexml_version"))
        .stdout(predicate::str::contains("type: GROUP"));
}

#[test]
fn dump_writes_to_output_file() {
    let out = temp_path("dump.yaml");
    tosc()
        .arg("dump")
        .arg(fixtures_dir().join("blank.tosc"))
        .arg("-o")
        .arg(&out)
        .assert()
        .success();
    let contents = std::fs::read_to_string(&out).unwrap();
    assert!(contents.contains("lexml_version"));
    std::fs::remove_file(&out).ok();
}

#[test]
fn dump_then_build_round_trips_a_fixture() {
    let yaml_path = temp_path("roundtrip.yaml");
    let tosc_path = temp_path("roundtrip.tosc");

    tosc()
        .arg("dump")
        .arg(fixtures_dir().join("multipage.tosc"))
        .arg("-o")
        .arg(&yaml_path)
        .assert()
        .success();

    tosc()
        .arg("build")
        .arg(&yaml_path)
        .arg("-o")
        .arg(&tosc_path)
        .assert()
        .success();

    let original =
        touch_osc_core::Layout::from_file(fixtures_dir().join("multipage.tosc")).unwrap();
    let rebuilt = touch_osc_core::Layout::from_bytes(&std::fs::read(&tosc_path).unwrap()).unwrap();
    assert_eq!(original, rebuilt);

    std::fs::remove_file(&yaml_path).ok();
    std::fs::remove_file(&tosc_path).ok();
}

#[test]
fn build_fails_on_validation_error_without_force() {
    let yaml_path = temp_path("bad.yaml");
    let tosc_path = temp_path("bad.tosc");
    std::fs::write(
        &yaml_path,
        r#"
lexml_version: '5'
root:
  id: 11111111-1111-1111-1111-111111111111
  type: GROUP
  properties:
  - type: i
    key: background
    value: 1
  - type: r
    key: frame
    value: {x: 0, y: 0, w: 450, h: 850}
  values: []
"#,
    )
    .unwrap();

    tosc()
        .arg("build")
        .arg(&yaml_path)
        .arg("-o")
        .arg(&tosc_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("validation error"));

    assert!(!tosc_path.exists());
    std::fs::remove_file(&yaml_path).ok();
}

#[test]
fn build_with_force_writes_despite_validation_errors() {
    let yaml_path = temp_path("bad-force.yaml");
    let tosc_path = temp_path("bad-force.tosc");
    std::fs::write(
        &yaml_path,
        r#"
lexml_version: '5'
root:
  id: 11111111-1111-1111-1111-111111111111
  type: GROUP
  properties:
  - type: i
    key: background
    value: 1
  - type: r
    key: frame
    value: {x: 0, y: 0, w: 450, h: 850}
  values: []
"#,
    )
    .unwrap();

    tosc()
        .arg("build")
        .arg(&yaml_path)
        .arg("-o")
        .arg(&tosc_path)
        .arg("--force")
        .assert()
        .success();

    assert!(tosc_path.exists());
    std::fs::remove_file(&yaml_path).ok();
    std::fs::remove_file(&tosc_path).ok();
}

#[test]
fn validate_exits_zero_on_a_clean_fixture_even_with_warnings() {
    // blank.tosc has a warning (missing 'name') but no errors; exit code
    // must stay 0 -- only errors should fail the command.
    tosc()
        .arg("validate")
        .arg(fixtures_dir().join("blank.tosc"))
        .assert()
        .success();
}

#[test]
fn validate_exits_nonzero_on_a_file_with_errors() {
    let tosc_path = temp_path("invalid.tosc");
    let mut layout = touch_osc_core::Layout::from_file(fixtures_dir().join("blank.tosc")).unwrap();
    layout.root.properties.retain(|p| p.key() != "frame");
    layout.to_file(&tosc_path).unwrap();

    tosc()
        .arg("validate")
        .arg(&tosc_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("frame"));

    std::fs::remove_file(&tosc_path).ok();
}
