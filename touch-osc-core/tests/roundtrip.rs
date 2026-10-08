//! Round-trip tests against the real fixtures in `tests/fixtures/` at the
//! workspace root. These compare parsed *structure*, not compressed bytes
//! (compression settings can differ while still being a valid zlib
//! stream), which is what "semantically identical" means for goal 1.

use touch_osc_core::{Layout, Property};

fn fixtures_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures")
}

fn fixture_paths() -> Vec<std::path::PathBuf> {
    let dir = fixtures_dir();
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("reading fixtures dir {dir:?}: {e}"))
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("tosc"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "no .tosc fixtures found in {dir:?}");
    paths
}

#[test]
fn every_fixture_round_trips_through_compression_and_xml() {
    for path in fixture_paths() {
        let layout = Layout::from_file(&path).unwrap_or_else(|e| panic!("parsing {path:?}: {e}"));

        let bytes = layout
            .to_bytes()
            .unwrap_or_else(|e| panic!("serializing {path:?}: {e}"));
        let layout2 = Layout::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("re-parsing serialized {path:?}: {e}"));

        assert_eq!(
            layout, layout2,
            "{path:?} did not round-trip to an identical structure"
        );
    }
}

#[test]
fn every_fixture_round_trip_is_stable_under_a_second_pass() {
    for path in fixture_paths() {
        let layout = Layout::from_file(&path).unwrap();
        let once = layout.to_bytes().unwrap();
        let reparsed = Layout::from_bytes(&once).unwrap();
        let twice = reparsed.to_bytes().unwrap();
        let reparsed_again = Layout::from_bytes(&twice).unwrap();
        assert_eq!(
            reparsed, reparsed_again,
            "{path:?} structure drifted on a second round-trip pass"
        );
    }
}

#[test]
fn unknown_property_survives_round_trip_untouched() {
    let path = fixtures_dir().join("blank.tosc");
    let mut layout = Layout::from_file(&path).unwrap();

    // Simulate a future/unknown property type this crate has never seen.
    let injected = Property {
        type_code: "x".to_string(),
        key_raw: "<![CDATA[futureProperty]]>".to_string(),
        value_raw: "<somethingUnmodeled><nested>42</nested></somethingUnmodeled>".to_string(),
    };
    layout.root.properties.push(injected.clone());

    let bytes = layout.to_bytes().unwrap();
    let reloaded = Layout::from_bytes(&bytes).unwrap();

    let found = reloaded
        .root
        .property("futureProperty")
        .expect("injected unknown property should survive save/load");
    assert_eq!(found.type_code, "x");
    assert_eq!(
        found.value_raw,
        "<somethingUnmodeled><nested>42</nested></somethingUnmodeled>"
    );
}

#[test]
fn complex_setup_has_expected_scripts_and_messages() {
    let layout = Layout::from_file(fixtures_dir().join("complex_setup.tosc")).unwrap();
    let scripts: Vec<_> = layout
        .root
        .walk()
        .filter_map(|n| n.property("script"))
        .collect();
    assert_eq!(scripts.len(), 7, "expected seven scripted nodes");
    let unique: std::collections::HashSet<_> =
        scripts.iter().map(|p| p.as_string().unwrap()).collect();
    assert_eq!(unique.len(), 2, "expected two distinct script bodies");

    let has_osc = layout.root.walk().any(|n| !n.messages.is_empty());
    assert!(has_osc, "expected at least one node with OSC messages");
}

#[test]
fn multipage_has_pager_with_page_children() {
    let layout = Layout::from_file(fixtures_dir().join("multipage.tosc")).unwrap();
    let pager = layout
        .root
        .walk()
        .find(|n| n.node_type == "PAGER")
        .expect("expected a PAGER node");
    assert!(
        pager.children.len() > 1,
        "expected the pager to have multiple page children"
    );
    assert!(pager
        .children
        .iter()
        .all(|c| c.node_type == "GROUP" && c.property("tabLabel").is_some()));
}
