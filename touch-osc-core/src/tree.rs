//! The generic, lossless tree model for a `.tosc` layout.
//!
//! This is the source of truth: everything the typed builder/validation
//! layers do is expressed in terms of these types, and nothing here is
//! allowed to silently drop data it doesn't recognize. See
//! `docs/FORMAT.md` for the on-disk shapes these types mirror.
//!
//! Property and value *contents* (the inner XML of a `<value>` element,
//! the body of a node-level `<values><value>`, and the body of a
//! `<midi>`/`<osc>` message) are stored as raw XML text, byte-identical to
//! what was read from the source document. This is what makes unknown or
//! version-specific shapes survive a load/save cycle untouched: the parser
//! never has to understand a shape in order to preserve it. Typed
//! accessors (`as_bool`, `as_color`, ...) and constructors (`Property::bool`,
//! `Property::color`, ...) interpret/produce that raw text on top, for the
//! builder and validation layers to use.

use std::fmt;

/// The root of a parsed layout: the `<lexml version='...'>` wrapper and its
/// single `<node>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    /// The `version` attribute of `<lexml>`, e.g. `"5"`. Stored verbatim
    /// and not interpreted — see docs/FORMAT.md.
    pub lexml_version: String,
    pub root: Node,
}

/// A `<node ID='...' type='...'>` element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub node_type: String,
    pub properties: Vec<Property>,
    pub values: Vec<ValueEntry>,
    /// Message bindings (MIDI/OSC), in source order. Empty means the
    /// `<messages>` tag is omitted entirely on write (see docs/FORMAT.md:
    /// the editor never writes an empty `<messages></messages>`).
    pub messages: Vec<Message>,
    /// Child nodes, in source order. Empty means `<children>` is omitted
    /// entirely on write, same rationale as `messages`.
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(node_type: impl Into<String>) -> Self {
        Node {
            id: crate::idgen::new_id(),
            node_type: node_type.into(),
            properties: Vec::new(),
            values: Vec::new(),
            messages: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn property(&self, key: &str) -> Option<&Property> {
        self.properties.iter().find(|p| p.key() == key)
    }

    pub fn property_mut(&mut self, key: &str) -> Option<&mut Property> {
        self.properties.iter_mut().find(|p| p.key() == key)
    }

    /// Insert or replace a property by key, preserving the position of an
    /// existing property with that key, or appending if new.
    pub fn set_property(&mut self, property: Property) {
        if let Some(existing) = self.property_mut(&property.key()) {
            *existing = property;
        } else {
            self.properties.push(property);
        }
    }

    pub fn value(&self, key: &str) -> Option<&ValueEntry> {
        self.values.iter().find(|v| v.key() == key)
    }

    /// Recursively visit this node and all descendants.
    pub fn walk<'a>(&'a self) -> impl Iterator<Item = &'a Node> + 'a {
        NodeWalk { stack: vec![self] }
    }
}

struct NodeWalk<'a> {
    stack: Vec<&'a Node>,
}

impl<'a> Iterator for NodeWalk<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        for child in node.children.iter().rev() {
            self.stack.push(child);
        }
        Some(node)
    }
}

/// A `<property type='X'><key>...</key><value>...</value></property>` entry.
///
/// `key_raw` and `value_raw` are the exact inner XML text of `<key>` and
/// `<value>` respectively (e.g. `<![CDATA[background]]>` or `1` or
/// `<r>0</r><g>0</g><b>0</b><a>1</a>`), preserved byte-for-byte from the
/// source. Use [`Property::key`] for the decoded key string, and the
/// `as_*`/typed constructors for interpreting or producing `value_raw`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    pub type_code: String,
    pub key_raw: String,
    pub value_raw: String,
}

impl Property {
    pub fn key(&self) -> String {
        crate::text::decode(&self.key_raw)
    }

    pub fn is_type(&self, code: &str) -> bool {
        self.type_code == code
    }

    // --- typed constructors -------------------------------------------

    pub fn bool(key: &str, value: bool) -> Self {
        Property {
            type_code: "b".into(),
            key_raw: crate::text::encode_cdata(key),
            value_raw: if value { "1" } else { "0" }.into(),
        }
    }

    pub fn int(key: &str, value: i64) -> Self {
        Property {
            type_code: "i".into(),
            key_raw: crate::text::encode_cdata(key),
            value_raw: value.to_string(),
        }
    }

    pub fn float(key: &str, value: f64) -> Self {
        Property {
            type_code: "f".into(),
            key_raw: crate::text::encode_cdata(key),
            value_raw: crate::text::format_float(value),
        }
    }

    pub fn string(key: &str, value: &str) -> Self {
        Property {
            type_code: "s".into(),
            key_raw: crate::text::encode_cdata(key),
            value_raw: crate::text::encode_cdata(value),
        }
    }

    pub fn color(key: &str, color: Color) -> Self {
        Property {
            type_code: "c".into(),
            key_raw: crate::text::encode_cdata(key),
            value_raw: color.to_raw(),
        }
    }

    pub fn rect(key: &str, rect: Rect) -> Self {
        Property {
            type_code: "r".into(),
            key_raw: crate::text::encode_cdata(key),
            value_raw: rect.to_raw(),
        }
    }

    // --- typed accessors -------------------------------------------

    pub fn as_bool(&self) -> Option<bool> {
        match self.value_raw.trim() {
            "1" => Some(true),
            "0" => Some(false),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        self.value_raw.trim().parse().ok()
    }

    pub fn as_float(&self) -> Option<f64> {
        self.value_raw.trim().parse().ok()
    }

    pub fn as_string(&self) -> Option<String> {
        Some(crate::text::decode(&self.value_raw))
    }

    pub fn as_color(&self) -> Option<Color> {
        Color::from_raw(&self.value_raw)
    }

    pub fn as_rect(&self) -> Option<Rect> {
        Rect::from_raw(&self.value_raw)
    }
}

/// An RGBA color, components in `0.0..=1.0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub const fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Color { r, g, b, a }
    }

    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);

    fn to_raw(self) -> String {
        format!(
            "<r>{}</r><g>{}</g><b>{}</b><a>{}</a>",
            crate::text::format_float(self.r),
            crate::text::format_float(self.g),
            crate::text::format_float(self.b),
            crate::text::format_float(self.a)
        )
    }

    fn from_raw(raw: &str) -> Option<Color> {
        let r = crate::text::extract_tag_f64(raw, "r")?;
        let g = crate::text::extract_tag_f64(raw, "g")?;
        let b = crate::text::extract_tag_f64(raw, "b")?;
        let a = crate::text::extract_tag_f64(raw, "a")?;
        Some(Color { r, g, b, a })
    }
}

/// A frame (position + size), in points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub const fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Rect { x, y, w, h }
    }

    fn to_raw(self) -> String {
        format!(
            "<x>{}</x><y>{}</y><w>{}</w><h>{}</h>",
            crate::text::format_float(self.x),
            crate::text::format_float(self.y),
            crate::text::format_float(self.w),
            crate::text::format_float(self.h)
        )
    }

    fn from_raw(raw: &str) -> Option<Rect> {
        let x = crate::text::extract_tag_f64(raw, "x")?;
        let y = crate::text::extract_tag_f64(raw, "y")?;
        let w = crate::text::extract_tag_f64(raw, "w")?;
        let h = crate::text::extract_tag_f64(raw, "h")?;
        Some(Rect { x, y, w, h })
    }
}

/// A node-level `<values><value>...</value></values>` entry describing an
/// interactive value the control exposes (`touch`, `x`, `y`, `text`,
/// `page`, ...). `body_raw` is the exact inner XML following `<key>`,
/// i.e. the `<locked>...</defaultPull>` portion, preserved byte-for-byte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueEntry {
    pub key_raw: String,
    pub body_raw: String,
}

impl ValueEntry {
    pub fn key(&self) -> String {
        crate::text::decode(&self.key_raw)
    }

    /// Build a value entry with the editor's standard defaults
    /// (`locked`/`lockedDefaultCurrent`/`defaultPull` all `0`), the shape
    /// observed on every value in every fixture.
    pub fn new(key: &str, default_raw: &str) -> Self {
        ValueEntry {
            key_raw: crate::text::encode_cdata(key),
            body_raw: format!(
                "<locked>0</locked><lockedDefaultCurrent>0</lockedDefaultCurrent><default>{default_raw}</default><defaultPull>0</defaultPull>"
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Midi,
    Osc,
}

impl fmt::Display for MessageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageKind::Midi => write!(f, "midi"),
            MessageKind::Osc => write!(f, "osc"),
        }
    }
}

/// A single `<midi>...</midi>` or `<osc>...</osc>` message binding.
/// `raw` is the exact inner XML, preserved byte-for-byte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub kind: MessageKind,
    pub raw: String,
}
