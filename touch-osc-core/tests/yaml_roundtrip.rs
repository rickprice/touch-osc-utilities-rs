//! `dump` -> `build` round-trip tests against every real fixture: goal 2
//! requires that converting a `.tosc` to YAML and back loses nothing.

use touch_osc_core::Layout;

fn fixtures_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures")
}

fn fixture_paths() -> Vec<std::path::PathBuf> {
    let dir = fixtures_dir();
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("tosc"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty());
    paths
}

#[test]
fn every_fixture_round_trips_through_yaml() {
    for path in fixture_paths() {
        let layout = Layout::from_file(&path).unwrap_or_else(|e| panic!("parsing {path:?}: {e}"));

        let yaml =
            touch_osc_core::yaml::dump(&layout).unwrap_or_else(|e| panic!("dumping {path:?}: {e}"));
        let rebuilt = touch_osc_core::yaml::build(&yaml)
            .unwrap_or_else(|e| panic!("building from dump of {path:?}: {e}"));

        assert_eq!(
            layout, rebuilt,
            "{path:?} did not round-trip through the YAML text representation"
        );

        // And the rebuilt layout must still produce a valid, re-parseable
        // .tosc (not just an equal in-memory structure).
        let bytes = rebuilt
            .to_bytes()
            .unwrap_or_else(|e| panic!("serializing rebuilt {path:?}: {e}"));
        let reloaded = Layout::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("re-parsing rebuilt {path:?}: {e}"));
        assert_eq!(layout, reloaded);
    }
}

#[test]
fn dump_only_uses_raw_xml_fallback_for_messages() {
    // Sanity check that normal fixtures don't fall back to a raw_xml
    // wrapper for property/value shapes this crate is supposed to
    // understand (b/i/f/s/c/r properties and standard value bodies).
    // `TextMessage` always has a field literally named `raw_xml` by
    // design (messages aren't decoded yet, see docs/FORMAT.md), so the
    // expected count is exactly the number of message bindings in the
    // layout -- anything beyond that means our verified-reconstruction
    // check is failing on real property/value data.
    for path in fixture_paths() {
        let layout = Layout::from_file(&path).unwrap();
        let expected_message_count: usize = layout.root.walk().map(|n| n.messages.len()).sum();

        let yaml = touch_osc_core::yaml::dump(&layout).unwrap();
        let raw_xml_count = yaml.matches("raw_xml").count();
        assert_eq!(
            raw_xml_count, expected_message_count,
            "{path:?}: expected raw_xml to appear only for the {expected_message_count} message binding(s)"
        );
    }
}
