//! Typed builders for common controls.
//!
//! Each constructor here (`group`, `button`, `fader`, ...) returns a
//! [`ControlBuilder`] pre-populated with the exact default property set
//! observed on that control type in `tests/fixtures/everything.tosc` (a
//! layout built in the real editor with one of each control type, kept
//! close to its freshly-added defaults) — not a guessed/minimal set. See
//! `docs/FORMAT.md` for the per-type property tables these mirror.
//!
//! This is a convenience layer on top of [`crate::tree::Node`], which
//! remains fully public: anything not covered by a fluent method here
//! (an unusual property, a type this module doesn't have a constructor
//! for) is reachable directly via `Node::set_property`/`.properties`/
//! `.children`/`.messages`, so the typed layer is never a dead end.

use crate::tree::{Color, Message, Node, Property, Rect, ValueEntry};

/// Wraps a [`Node`] under construction with fluent setters shared by
/// every control type.
pub struct ControlBuilder(Node);

impl ControlBuilder {
    fn new(node_type: &str) -> Self {
        ControlBuilder(Node::new(node_type))
    }

    pub fn frame(mut self, frame: Rect) -> Self {
        self.0.set_property(Property::rect("frame", frame));
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.0.set_property(Property::color("color", color));
        self
    }

    pub fn background(mut self, visible: bool) -> Self {
        self.0.set_property(Property::bool("background", visible));
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.0.set_property(Property::bool("visible", visible));
        self
    }

    pub fn locked(mut self, locked: bool) -> Self {
        self.0.set_property(Property::bool("locked", locked));
        self
    }

    pub fn interactive(mut self, interactive: bool) -> Self {
        self.0
            .set_property(Property::bool("interactive", interactive));
        self
    }

    pub fn outline(mut self, outline: bool) -> Self {
        self.0.set_property(Property::bool("outline", outline));
        self
    }

    pub fn corner_radius(mut self, radius: f64) -> Self {
        self.0.set_property(Property::float("cornerRadius", radius));
        self
    }

    /// Set an arbitrary property, overriding any default with the same
    /// key. Escape hatch for anything not covered by a named method.
    pub fn property(mut self, property: Property) -> Self {
        self.0.set_property(property);
        self
    }

    pub fn child(mut self, child: impl Into<Node>) -> Self {
        self.0.children.push(child.into());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = Node>) -> Self {
        self.0.children.extend(children);
        self
    }

    pub fn message(mut self, message: Message) -> Self {
        self.0.messages.push(message);
        self
    }

    pub fn messages(mut self, messages: impl IntoIterator<Item = Message>) -> Self {
        self.0.messages.extend(messages);
        self
    }

    pub fn build(self) -> Node {
        self.0
    }
}

impl From<ControlBuilder> for Node {
    fn from(b: ControlBuilder) -> Node {
        b.build()
    }
}

/// A plain container. Also the type used for a `PAGER`'s page children
/// (see [`page`]) and, with no name, a top-level layout canvas.
pub fn group(name: &str) -> ControlBuilder {
    let mut b = ControlBuilder::new("GROUP");
    b.0.properties = vec![
        Property::bool("background", true),
        Property::color("color", Color::BLACK),
        Property::float("cornerRadius", 1.0),
        Property::rect("frame", Rect::new(0.0, 0.0, 450.0, 850.0)),
        Property::bool("grabFocus", false),
        Property::bool("interactive", false),
        Property::bool("locked", false),
        Property::string("name", name),
        Property::int("orientation", 0),
        Property::bool("outline", true),
        Property::int("outlineStyle", 0),
        Property::int("pointerPriority", 0),
        Property::int("shape", 1),
        Property::bool("visible", true),
    ];
    b.0.values
        .push(ValueEntry::new("touch", "<![CDATA[false]]>"));
    b
}

/// A `GROUP` meant to be a direct child of a [`pager`], with the tab
/// chrome properties the editor adds to page containers.
pub fn page(name: &str, tab_label: &str) -> ControlBuilder {
    let mut b = group(name);
    b.0.properties.extend([
        Property::color("tabColorOff", Color::new(0.25, 0.25, 0.25, 1.0)),
        Property::color("tabColorOn", Color::new(0.0, 0.0, 0.0, 0.0)),
        Property::string("tabLabel", tab_label),
        Property::color("textColorOff", Color::WHITE),
        Property::color("textColorOn", Color::WHITE),
    ]);
    // tabColorOff/tabColorOn/tabLabel sort alphabetically right after
    // `shape` and before `textColorOff`/`visible`; re-sort to match the
    // editor's alphabetical property order (see docs/FORMAT.md).
    b.0.properties.sort_by_key(|p| p.key());
    b
}

pub fn pager(name: &str) -> ControlBuilder {
    let mut b = ControlBuilder::new("PAGER");
    b.0.properties = vec![
        Property::bool("background", true),
        Property::color("color", Color::new(0.25, 0.25, 0.25, 1.0)),
        Property::float("cornerRadius", 1.0),
        Property::rect("frame", Rect::new(10.0, 10.0, 240.0, 240.0)),
        Property::bool("grabFocus", false),
        Property::bool("interactive", true),
        Property::bool("locked", false),
        Property::string("name", name),
        Property::int("orientation", 0),
        Property::bool("outline", true),
        Property::int("outlineStyle", 0),
        Property::int("pointerPriority", 0),
        Property::int("shape", 1),
        Property::bool("tabLabels", true),
        Property::bool("tabbar", true),
        Property::bool("tabbarDoubleTap", false),
        Property::int("tabbarSize", 40),
        Property::int("textSizeOff", 14),
        Property::int("textSizeOn", 14),
        Property::bool("visible", true),
    ];
    b.0.values.push(ValueEntry::new("page", "<![CDATA[0]]>"));
    b.0.values
        .push(ValueEntry::new("touch", "<![CDATA[false]]>"));
    b
}

pub fn box_control(name: &str) -> ControlBuilder {
    let mut b = ControlBuilder::new("BOX");
    b.0.properties = vec![
        Property::bool("background", true),
        Property::color("color", Color::new(1.0, 0.0, 0.0, 1.0)),
        Property::float("cornerRadius", 1.0),
        Property::rect("frame", Rect::new(10.0, 10.0, 50.0, 50.0)),
        Property::bool("grabFocus", true),
        Property::bool("interactive", false),
        Property::bool("locked", false),
        Property::string("name", name),
        Property::int("orientation", 0),
        Property::bool("outline", false),
        Property::int("outlineStyle", 1),
        Property::int("pointerPriority", 0),
        Property::int("shape", 1),
        Property::bool("visible", true),
    ];
    b.0.values
        .push(ValueEntry::new("touch", "<![CDATA[false]]>"));
    b
}

pub fn button(name: &str) -> ControlBuilder {
    let mut b = ControlBuilder::new("BUTTON");
    b.0.properties = vec![
        Property::bool("background", true),
        Property::int("buttonType", 0),
        Property::color("color", Color::new(1.0, 0.0, 0.0, 1.0)),
        Property::float("cornerRadius", 1.0),
        Property::rect("frame", Rect::new(10.0, 10.0, 50.0, 50.0)),
        Property::bool("grabFocus", true),
        Property::bool("interactive", true),
        Property::bool("locked", false),
        Property::string("name", name),
        Property::int("orientation", 0),
        Property::bool("outline", true),
        Property::int("outlineStyle", 1),
        Property::int("pointerPriority", 0),
        Property::bool("press", true),
        Property::bool("release", true),
        Property::int("shape", 1),
        Property::bool("valuePosition", false),
        Property::bool("visible", true),
    ];
    b.0.values.push(ValueEntry::new("x", "<![CDATA[0]]>"));
    b.0.values
        .push(ValueEntry::new("touch", "<![CDATA[false]]>"));
    b
}

pub fn label(name: &str, text: &str) -> ControlBuilder {
    let mut b = ControlBuilder::new("LABEL");
    b.0.properties = vec![
        Property::bool("background", true),
        Property::color("color", Color::new(1.0, 0.0, 0.0, 1.0)),
        Property::float("cornerRadius", 1.0),
        Property::int("font", 0),
        Property::rect("frame", Rect::new(10.0, 10.0, 80.0, 25.0)),
        Property::bool("grabFocus", true),
        Property::bool("interactive", false),
        Property::bool("locked", false),
        Property::string("name", name),
        Property::int("orientation", 0),
        Property::bool("outline", true),
        Property::int("outlineStyle", 1),
        Property::int("pointerPriority", 0),
        Property::int("shape", 1),
        Property::int("textAlignH", 2),
        Property::int("textAlignV", 2),
        Property::bool("textClip", true),
        Property::color("textColor", Color::WHITE),
        Property::int("textLength", 0),
        Property::int("textSize", 14),
        Property::bool("visible", true),
    ];
    b.0.values
        .push(ValueEntry::new("text", &crate::text::encode_cdata(text)));
    b.0.values
        .push(ValueEntry::new("touch", "<![CDATA[false]]>"));
    b
}

pub fn text_field(name: &str, text: &str) -> ControlBuilder {
    let mut b = ControlBuilder::new("TEXT");
    b.0.properties = vec![
        Property::bool("background", true),
        Property::color("color", Color::new(1.0, 0.0, 0.0, 1.0)),
        Property::float("cornerRadius", 1.0),
        Property::int("font", 0),
        Property::rect("frame", Rect::new(10.0, 10.0, 240.0, 100.0)),
        Property::bool("grabFocus", true),
        Property::bool("interactive", false),
        Property::bool("locked", false),
        Property::string("name", name),
        Property::int("orientation", 0),
        Property::bool("outline", true),
        Property::int("outlineStyle", 1),
        Property::int("pointerPriority", 0),
        Property::int("shape", 1),
        Property::int("textAlignH", 1),
        Property::int("textAlignV", 1),
        Property::bool("textClip", true),
        Property::color("textColor", Color::WHITE),
        Property::int("textSize", 14),
        Property::bool("textWrap", true),
        Property::bool("visible", true),
    ];
    b.0.values
        .push(ValueEntry::new("text", &crate::text::encode_cdata(text)));
    b.0.values
        .push(ValueEntry::new("touch", "<![CDATA[false]]>"));
    b
}

pub fn fader(name: &str) -> ControlBuilder {
    let mut b = ControlBuilder::new("FADER");
    b.0.properties = vec![
        Property::bool("background", true),
        Property::bool("bar", true),
        Property::int("barDisplay", 0),
        Property::color("color", Color::new(1.0, 0.0, 0.0, 1.0)),
        Property::float("cornerRadius", 1.0),
        Property::bool("cursor", true),
        Property::int("cursorDisplay", 0),
        Property::rect("frame", Rect::new(10.0, 10.0, 50.0, 200.0)),
        Property::bool("grabFocus", true),
        Property::bool("grid", true),
        Property::color("gridColor", Color::new(0.0, 0.0, 0.0, 0.25)),
        Property::int("gridSteps", 13),
        Property::bool("interactive", true),
        Property::bool("locked", false),
        Property::string("name", name),
        Property::int("orientation", 0),
        Property::bool("outline", true),
        Property::int("outlineStyle", 1),
        Property::int("pointerPriority", 0),
        Property::int("response", 0),
        Property::int("responseFactor", 100),
        Property::int("shape", 1),
        Property::bool("visible", true),
    ];
    b.0.values.push(ValueEntry::new("x", "<![CDATA[0]]>"));
    b.0.values
        .push(ValueEntry::new("touch", "<![CDATA[false]]>"));
    b
}

pub fn xy_pad(name: &str) -> ControlBuilder {
    let mut b = ControlBuilder::new("XY");
    b.0.properties = vec![
        Property::bool("background", true),
        Property::color("color", Color::new(1.0, 0.0, 0.0, 1.0)),
        Property::float("cornerRadius", 1.0),
        Property::bool("cursor", true),
        Property::int("cursorDisplay", 0),
        Property::rect("frame", Rect::new(10.0, 10.0, 240.0, 240.0)),
        Property::bool("grabFocus", true),
        Property::color("gridColor", Color::new(0.0, 0.0, 0.0, 0.25)),
        Property::int("gridStepsX", 10),
        Property::int("gridStepsY", 10),
        Property::bool("gridX", true),
        Property::bool("gridY", true),
        Property::bool("interactive", true),
        Property::bool("lines", true),
        Property::int("linesDisplay", 0),
        Property::bool("lockX", false),
        Property::bool("lockY", false),
        Property::bool("locked", false),
        Property::string("name", name),
        Property::int("orientation", 0),
        Property::bool("outline", true),
        Property::int("outlineStyle", 1),
        Property::int("pointerPriority", 0),
        Property::int("response", 0),
        Property::int("responseFactor", 100),
        Property::int("shape", 1),
        Property::bool("visible", true),
    ];
    b.0.values.push(ValueEntry::new("x", "<![CDATA[0]]>"));
    b.0.values.push(ValueEntry::new("y", "<![CDATA[0]]>"));
    b.0.values
        .push(ValueEntry::new("touch", "<![CDATA[false]]>"));
    b
}

pub fn radial(name: &str) -> ControlBuilder {
    let mut b = ControlBuilder::new("RADIAL");
    b.0.properties = vec![
        Property::bool("background", true),
        Property::bool("centered", false),
        Property::color("color", Color::new(1.0, 0.0, 0.0, 1.0)),
        Property::float("cornerRadius", 1.0),
        Property::rect("frame", Rect::new(10.0, 10.0, 150.0, 150.0)),
        Property::bool("grabFocus", true),
        Property::bool("grid", true),
        Property::color("gridColor", Color::new(0.0, 0.0, 0.0, 0.25)),
        Property::int("gridSteps", 20),
        Property::bool("interactive", true),
        Property::bool("inverted", false),
        Property::bool("locked", false),
        Property::string("name", name),
        Property::int("orientation", 0),
        Property::bool("outline", true),
        Property::int("outlineStyle", 0),
        Property::int("pointerPriority", 0),
        Property::int("response", 0),
        Property::int("responseFactor", 100),
        Property::int("shape", 2),
        Property::bool("visible", true),
    ];
    b.0.values.push(ValueEntry::new("x", "<![CDATA[0]]>"));
    b.0.values
        .push(ValueEntry::new("touch", "<![CDATA[false]]>"));
    b
}

pub fn encoder(name: &str) -> ControlBuilder {
    let mut b = ControlBuilder::new("ENCODER");
    b.0.properties = vec![
        Property::bool("background", true),
        Property::color("color", Color::new(1.0, 0.0, 0.0, 1.0)),
        Property::float("cornerRadius", 1.0),
        Property::bool("cursor", true),
        Property::int("cursorDisplay", 0),
        Property::rect("frame", Rect::new(10.0, 10.0, 150.0, 150.0)),
        Property::bool("grabFocus", true),
        Property::bool("grid", true),
        Property::color("gridColor", Color::new(0.0, 0.0, 0.0, 0.25)),
        Property::int("gridSteps", 20),
        Property::bool("interactive", true),
        Property::bool("locked", false),
        Property::string("name", name),
        Property::int("orientation", 0),
        Property::bool("outline", true),
        Property::int("outlineStyle", 0),
        Property::int("pointerPriority", 0),
        Property::int("response", 0),
        Property::int("responseFactor", 100),
        Property::int("shape", 2),
        Property::bool("visible", true),
    ];
    b.0.values.push(ValueEntry::new("x", "<![CDATA[0]]>"));
    b.0.values.push(ValueEntry::new("y", "<![CDATA[0]]>"));
    b.0.values
        .push(ValueEntry::new("touch", "<![CDATA[false]]>"));
    b
}

pub fn radar(name: &str) -> ControlBuilder {
    let mut b = ControlBuilder::new("RADAR");
    b.0.properties = vec![
        Property::bool("background", true),
        Property::color("color", Color::new(1.0, 0.0, 0.0, 1.0)),
        Property::float("cornerRadius", 1.0),
        Property::bool("cursor", true),
        Property::int("cursorDisplay", 0),
        Property::rect("frame", Rect::new(10.0, 10.0, 240.0, 240.0)),
        Property::bool("grabFocus", true),
        Property::color("gridColor", Color::new(0.0, 0.0, 0.0, 0.25)),
        Property::int("gridStepsX", 10),
        Property::int("gridStepsY", 10),
        Property::bool("gridX", true),
        Property::bool("gridY", true),
        Property::bool("interactive", true),
        Property::bool("lines", true),
        Property::int("linesDisplay", 0),
        Property::bool("lockX", false),
        Property::bool("lockY", false),
        Property::bool("locked", false),
        Property::string("name", name),
        Property::int("orientation", 0),
        Property::bool("outline", true),
        Property::int("outlineStyle", 0),
        Property::int("pointerPriority", 0),
        Property::int("shape", 2),
        Property::bool("visible", true),
    ];
    b.0.values.push(ValueEntry::new("x", "<![CDATA[0]]>"));
    b.0.values.push(ValueEntry::new("y", "<![CDATA[0]]>"));
    b.0.values
        .push(ValueEntry::new("touch", "<![CDATA[false]]>"));
    b
}

pub fn radio(name: &str, steps: i64) -> ControlBuilder {
    let mut b = ControlBuilder::new("RADIO");
    b.0.properties = vec![
        Property::bool("background", true),
        Property::color("color", Color::new(1.0, 0.0, 0.0, 1.0)),
        Property::float("cornerRadius", 1.0),
        Property::rect("frame", Rect::new(10.0, 10.0, 200.0, 50.0)),
        Property::bool("grabFocus", true),
        Property::bool("interactive", true),
        Property::bool("locked", false),
        Property::string("name", name),
        Property::int("orientation", 1),
        Property::bool("outline", true),
        Property::int("outlineStyle", 1),
        Property::int("pointerPriority", 0),
        Property::int("radioType", 0),
        Property::int("shape", 1),
        Property::int("steps", steps),
        Property::bool("visible", true),
    ];
    b.0.values.push(ValueEntry::new("x", "<![CDATA[0]]>"));
    b.0.values
        .push(ValueEntry::new("touch", "<![CDATA[false]]>"));
    b
}

pub fn grid(name: &str, columns: i64, rows: i64) -> ControlBuilder {
    let mut b = ControlBuilder::new("GRID");
    b.0.properties = vec![
        Property::bool("background", true),
        Property::color("color", Color::new(1.0, 0.0, 0.0, 1.0)),
        Property::float("cornerRadius", 1.0),
        Property::bool("exclusive", false),
        Property::rect("frame", Rect::new(10.0, 10.0, 240.0, 240.0)),
        Property::bool("grabFocus", true),
        Property::int("gridNaming", 0),
        Property::int("gridOrder", 0),
        Property::int("gridStart", 0),
        Property::int("gridType", 4),
        Property::int("gridX", columns),
        Property::int("gridY", rows),
        Property::bool("interactive", true),
        Property::bool("locked", false),
        Property::string("name", name),
        Property::int("orientation", 0),
        Property::bool("outline", true),
        Property::int("outlineStyle", 1),
        Property::int("pointerPriority", 0),
        Property::int("shape", 1),
        Property::bool("visible", true),
    ];
    b.0.values
        .push(ValueEntry::new("touch", "<![CDATA[false]]>"));
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_has_sensible_defaults_and_overrides() {
        let node: Node = button("play")
            .frame(Rect::new(0.0, 0.0, 100.0, 40.0))
            .color(Color::new(0.0, 0.5, 1.0, 1.0))
            .build();
        assert_eq!(node.node_type, "BUTTON");
        assert_eq!(node.property("name").unwrap().as_string().unwrap(), "play");
        assert_eq!(
            node.property("frame").unwrap().as_rect().unwrap(),
            Rect::new(0.0, 0.0, 100.0, 40.0)
        );
        assert_eq!(
            node.property("color").unwrap().as_color().unwrap(),
            Color::new(0.0, 0.5, 1.0, 1.0)
        );
        assert_eq!(node.property("buttonType").unwrap().as_int(), Some(0));
        assert!(!node.id.is_empty());
    }

    #[test]
    fn group_nests_children_via_builder() {
        let node: Node = group("root").child(button("a")).child(button("b")).build();
        assert_eq!(node.children.len(), 2);
        assert_eq!(node.children[0].property("name").unwrap().key(), "name");
    }

    #[test]
    fn page_has_tab_chrome_and_sorted_properties() {
        let node: Node = page("page1", "Page 1").build();
        assert!(node.property("tabLabel").is_some());
        let keys: Vec<_> = node.properties.iter().map(|p| p.key()).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
    }

    #[test]
    fn every_builder_produces_unique_ids() {
        let a = button("a").build();
        let b = button("b").build();
        assert_ne!(a.id, b.id);
    }
}
