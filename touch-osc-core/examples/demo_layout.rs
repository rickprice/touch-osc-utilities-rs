//! Writes a layout built entirely from the typed `controls`/`messages`
//! builders to a `.tosc` file, for manual verification in the real
//! TouchOSC editor. Run with:
//!
//!     cargo run -p touch-osc-core --example demo_layout -- /path/to/out.tosc

use touch_osc_core::controls;
use touch_osc_core::messages::{midi_cc_binding, osc_binding, Conversion};
use touch_osc_core::tree::{Color, Layout, Node, Rect};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "demo_layout.tosc".to_string());

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

    let layout = Layout {
        lexml_version: "5".to_string(),
        root,
    };

    layout.to_file(&out_path)?;
    println!("wrote {out_path}");
    Ok(())
}
