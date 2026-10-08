//! End-to-end check that a layout built entirely from the typed
//! `controls`/`messages` builders (no hand-written XML, no loaded
//! fixture) round-trips through compression and through the YAML text
//! representation, and produces no duplicate node IDs.

use touch_osc_core::messages::{midi_cc_binding, osc_binding, Conversion};
use touch_osc_core::tree::{Color, Layout, Node, Rect};
use touch_osc_core::{controls, yaml};

fn demo_layout() -> Layout {
    let page1 = controls::page("page1", "Mixer")
        .child(
            controls::fader("level")
                .frame(Rect::new(20.0, 20.0, 60.0, 200.0))
                .message(osc_binding("x", Conversion::Float, true))
                .message(midi_cc_binding("x", 0, 127.0)),
        )
        .child(controls::label("levelLabel", "Level").frame(Rect::new(20.0, 230.0, 60.0, 20.0)))
        .build();

    let page2 = controls::page("page2", "Pad")
        .child(
            controls::button("trigger")
                .frame(Rect::new(20.0, 20.0, 50.0, 50.0))
                .message(osc_binding("x", Conversion::Float, true)),
        )
        .child(
            controls::xy_pad("position")
                .frame(Rect::new(90.0, 20.0, 150.0, 150.0))
                .message(osc_binding("x", Conversion::Float, true))
                .message(osc_binding("y", Conversion::Float, true)),
        )
        .build();

    let pager = controls::pager("pager1")
        .frame(Rect::new(0.0, 0.0, 300.0, 300.0))
        .message(osc_binding("page", Conversion::Integer, false))
        .message(midi_cc_binding("page", 0, 1.0))
        .child(page1)
        .child(page2)
        .build();

    let root: Node = controls::group("demo")
        .frame(Rect::new(0.0, 0.0, 300.0, 300.0))
        .color(Color::BLACK)
        .child(pager)
        .build();

    Layout {
        lexml_version: "5".to_string(),
        root,
    }
}

#[test]
fn builder_demo_layout_has_unique_ids() {
    let layout = demo_layout();
    let mut ids: Vec<_> = layout.root.walk().map(|n| n.id.clone()).collect();
    let before = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), before, "builder produced duplicate node IDs");
}

#[test]
fn builder_demo_layout_round_trips_through_compression() {
    let layout = demo_layout();
    let bytes = layout.to_bytes().unwrap();
    let reloaded = Layout::from_bytes(&bytes).unwrap();
    assert_eq!(layout, reloaded);
}

#[test]
fn builder_demo_layout_round_trips_through_yaml() {
    let layout = demo_layout();
    let text = yaml::dump(&layout).unwrap();
    let rebuilt = yaml::build(&text).unwrap();
    assert_eq!(layout, rebuilt);
}

#[test]
fn builder_demo_layout_has_two_pages_under_pager() {
    let layout = demo_layout();
    let pager = layout.root.walk().find(|n| n.node_type == "PAGER").unwrap();
    assert_eq!(pager.children.len(), 2);
    assert!(pager.children.iter().all(|c| c.node_type == "GROUP"));
}
