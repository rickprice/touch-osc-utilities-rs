//! The diffable YAML text representation used by `tosc dump`/`tosc build`.
//!
//! YAML was chosen over JSON for one reason that matters a lot in
//! practice: Lua scripts. A `script` property's value is multi-line Lua
//! source; as YAML it renders as a literal block scalar (plain indented
//! text), while as JSON it would be a single line with every newline
//! escaped as `\n` — unreadable and undiffable, defeating the point of
//! goal 2. YAML also tolerates the structural edits this format expects
//! (reordering a list item, adding a property) without the comma/bracket
//! bookkeeping JSON needs.
//!
//! Known property/value shapes (bool, int, float, string, color, rect,
//! the standard `locked`/`lockedDefaultCurrent`/`default`/`defaultPull`
//! value-entry body) are rendered as native YAML for readability. Before
//! doing so, the conversion is verified by reconstructing the raw XML
//! from the decoded value and comparing it byte-for-byte against the
//! original. If it doesn't match — an unusual encoding, a type code this
//! crate doesn't know, a future shape — the entry falls back to a
//! `{ raw_xml: "..." }` wrapper instead of silently normalizing data it
//! doesn't fully understand. This is what lets `tosc build` reproduce
//! the original file exactly even for data this crate can't interpret,
//! matching the generic tree's own "never discard unknown data" rule.
//!
//! One `s`-type property value gets an extra input form on `build`: a
//! `{ file: "path/to/script.lua" }` mapping, resolved relative to
//! `base_dir` and inlined as the property's string value. This is how
//! goal 5 ("keep scripts as separate `.lua` files, inlined at build
//! time") is implemented — `.tosc` itself has no concept of external
//! files (see docs/FORMAT.md), so this only exists on the way in; `dump`
//! always inlines the actual script text, since there's no original file
//! path to point back to once a layout has been loaded from a `.tosc`.

use crate::error::{Error, Result};
use crate::text;
use crate::tree::{Color, Layout, Message, MessageKind, Node, Property, Rect, ValueEntry};
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::path::Path;

pub fn dump(layout: &Layout) -> Result<String> {
    let text_layout = TextLayout::from_layout(layout);
    Ok(serde_yaml::to_string(&text_layout)?)
}

/// Build a [`Layout`] from YAML, resolving any `{file: ...}` script
/// references relative to the current directory. Prefer
/// [`build_with_base`] when the YAML came from a file, so references are
/// resolved relative to that file's own directory instead.
pub fn build(yaml: &str) -> Result<Layout> {
    build_with_base(yaml, Path::new("."))
}

/// Build a [`Layout`] from YAML, resolving any `{file: ...}` script
/// references relative to `base_dir`.
pub fn build_with_base(yaml: &str, base_dir: &Path) -> Result<Layout> {
    let text_layout: TextLayout = serde_yaml::from_str(yaml)?;
    text_layout.into_layout(base_dir)
}

#[derive(Serialize, Deserialize)]
pub struct TextLayout {
    pub lexml_version: String,
    pub root: TextNode,
}

impl TextLayout {
    fn from_layout(layout: &Layout) -> Self {
        TextLayout {
            lexml_version: layout.lexml_version.clone(),
            root: TextNode::from_node(&layout.root),
        }
    }

    fn into_layout(self, base_dir: &Path) -> Result<Layout> {
        Ok(Layout {
            lexml_version: self.lexml_version,
            root: self.root.into_node(base_dir)?,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct TextNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<TextProperty>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<TextValue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<TextMessage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TextNode>,
}

impl TextNode {
    fn from_node(node: &Node) -> Self {
        TextNode {
            id: node.id.clone(),
            node_type: node.node_type.clone(),
            properties: node
                .properties
                .iter()
                .map(TextProperty::from_property)
                .collect(),
            values: node
                .values
                .iter()
                .map(TextValue::from_value_entry)
                .collect(),
            messages: node
                .messages
                .iter()
                .map(TextMessage::from_message)
                .collect(),
            children: node.children.iter().map(TextNode::from_node).collect(),
        }
    }

    fn into_node(self, base_dir: &Path) -> Result<Node> {
        Ok(Node {
            id: self.id,
            node_type: self.node_type,
            properties: self
                .properties
                .into_iter()
                .map(|p| p.into_property(base_dir))
                .collect::<Result<Vec<_>>>()?,
            values: self
                .values
                .into_iter()
                .map(TextValue::into_value_entry)
                .collect::<Result<Vec<_>>>()?,
            messages: self
                .messages
                .into_iter()
                .map(TextMessage::into_message)
                .collect::<Result<Vec<_>>>()?,
            children: self
                .children
                .into_iter()
                .map(|c| c.into_node(base_dir))
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

/// Properties are kept as an ordered list of `{type, key, value}` entries
/// rather than a `{key: value}` mapping. A mapping would be prettier, but
/// it can't unambiguously carry the `type` code a bare YAML scalar loses
/// (e.g. a `f`-type property holding the whole number `1` is otherwise
/// indistinguishable from an `i`-type property holding `1`) — and getting
/// that wrong would silently corrupt the type on build, which is worse
/// than the extra verbosity.
#[derive(Serialize, Deserialize)]
pub struct TextProperty {
    #[serde(rename = "type")]
    pub type_code: String,
    pub key: String,
    pub value: Value,
}

impl TextProperty {
    fn from_property(p: &Property) -> Self {
        TextProperty {
            type_code: p.type_code.clone(),
            key: p.key(),
            value: encode_property_value(p),
        }
    }

    fn into_property(self, base_dir: &Path) -> Result<Property> {
        if let Some(raw) = extract_raw_xml(&self.value) {
            return Ok(Property {
                type_code: self.type_code,
                key_raw: text::encode_cdata(&self.key),
                value_raw: raw,
            });
        }
        if let Some(file) = extract_script_file(&self.value) {
            if self.type_code != "s" {
                return Err(Error::Malformed(format!(
                    "property '{}' uses {{file: ...}} but has type '{}', not 's'",
                    self.key, self.type_code
                )));
            }
            let path = base_dir.join(&file);
            let contents = std::fs::read_to_string(&path).map_err(|e| {
                Error::Malformed(format!(
                    "reading script file {} for property '{}': {e}",
                    path.display(),
                    self.key
                ))
            })?;
            return Ok(Property::string(&self.key, &contents));
        }
        Ok(match self.type_code.as_str() {
            "b" => Property::bool(&self.key, self.value.as_bool().unwrap_or(false)),
            "i" => Property::int(&self.key, self.value.as_i64().unwrap_or(0)),
            "f" => Property::float(&self.key, self.value.as_f64().unwrap_or(0.0)),
            "s" => Property::string(&self.key, self.value.as_str().unwrap_or("")),
            "c" => Property::color(
                &self.key,
                mapping_to_color(&self.value).unwrap_or(Color::BLACK),
            ),
            "r" => Property::rect(
                &self.key,
                mapping_to_rect(&self.value).unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0)),
            ),
            other => {
                // Unknown type code and no raw_xml wrapper: only sensible
                // if the user wrote the literal value text directly.
                let value_raw = self.value.as_str().unwrap_or_default().to_string();
                Property {
                    type_code: other.to_string(),
                    key_raw: text::encode_cdata(&self.key),
                    value_raw,
                }
            }
        })
    }
}

fn extract_script_file(v: &Value) -> Option<String> {
    let m = v.as_mapping()?;
    if m.len() == 1 {
        if let Some(Value::String(s)) = m.get(Value::String("file".to_string())) {
            return Some(s.clone());
        }
    }
    None
}

fn encode_property_value(p: &Property) -> Value {
    match p.type_code.as_str() {
        "b" => {
            if let Some(b) = p.as_bool() {
                if Property::bool(&p.key(), b).value_raw == p.value_raw {
                    return Value::Bool(b);
                }
            }
        }
        "i" => {
            if let Some(i) = p.as_int() {
                if Property::int(&p.key(), i).value_raw == p.value_raw {
                    return Value::Number(i.into());
                }
            }
        }
        "f" => {
            if let Some(f) = p.as_float() {
                if Property::float(&p.key(), f).value_raw == p.value_raw {
                    return Value::Number(f.into());
                }
            }
        }
        "s" => {
            if let Some(s) = p.as_string() {
                if Property::string(&p.key(), &s).value_raw == p.value_raw {
                    return Value::String(s);
                }
            }
        }
        "c" => {
            if let Some(c) = p.as_color() {
                if Property::color(&p.key(), c).value_raw == p.value_raw {
                    return color_to_value(c);
                }
            }
        }
        "r" => {
            if let Some(r) = p.as_rect() {
                if Property::rect(&p.key(), r).value_raw == p.value_raw {
                    return rect_to_value(r);
                }
            }
        }
        _ => {}
    }
    raw_xml_value(&p.value_raw)
}

fn color_to_value(c: Color) -> Value {
    let mut m = Mapping::new();
    m.insert("r".into(), c.r.into());
    m.insert("g".into(), c.g.into());
    m.insert("b".into(), c.b.into());
    m.insert("a".into(), c.a.into());
    Value::Mapping(m)
}

fn rect_to_value(r: Rect) -> Value {
    let mut m = Mapping::new();
    m.insert("x".into(), r.x.into());
    m.insert("y".into(), r.y.into());
    m.insert("w".into(), r.w.into());
    m.insert("h".into(), r.h.into());
    Value::Mapping(m)
}

fn mapping_to_color(v: &Value) -> Option<Color> {
    let m = v.as_mapping()?;
    Some(Color::new(
        get_f64(m, "r")?,
        get_f64(m, "g")?,
        get_f64(m, "b")?,
        get_f64(m, "a")?,
    ))
}

fn mapping_to_rect(v: &Value) -> Option<Rect> {
    let m = v.as_mapping()?;
    Some(Rect::new(
        get_f64(m, "x")?,
        get_f64(m, "y")?,
        get_f64(m, "w")?,
        get_f64(m, "h")?,
    ))
}

fn get_f64(m: &Mapping, key: &str) -> Option<f64> {
    m.get(Value::String(key.to_string()))?.as_f64()
}

fn raw_xml_value(raw: &str) -> Value {
    let mut m = Mapping::new();
    m.insert("raw_xml".into(), Value::String(raw.to_string()));
    Value::Mapping(m)
}

fn extract_raw_xml(v: &Value) -> Option<String> {
    let m = v.as_mapping()?;
    if m.len() == 1 {
        if let Some(Value::String(s)) = m.get(Value::String("raw_xml".to_string())) {
            return Some(s.clone());
        }
    }
    None
}

/// A node-level `<values><value>` entry. See [`crate::tree::ValueEntry`].
#[derive(Serialize, Deserialize)]
pub struct TextValue {
    pub key: String,
    pub body: Value,
}

impl TextValue {
    fn from_value_entry(v: &ValueEntry) -> Self {
        TextValue {
            key: v.key(),
            body: encode_value_body(v),
        }
    }

    fn into_value_entry(self) -> Result<ValueEntry> {
        if let Some(raw) = extract_raw_xml(&self.body) {
            return Ok(ValueEntry {
                key_raw: text::encode_cdata(&self.key),
                body_raw: raw,
            });
        }
        let parsed = ParsedValueBody::from_yaml(&self.body).ok_or_else(|| {
            Error::Malformed(format!(
                "value '{}' has an unrecognized body shape (expected locked/lockedDefaultCurrent/default/defaultPull, or {{raw_xml: ...}})",
                self.key
            ))
        })?;
        Ok(ValueEntry {
            key_raw: text::encode_cdata(&self.key),
            body_raw: parsed.to_raw(),
        })
    }
}

struct ParsedValueBody {
    locked: bool,
    locked_default_current: bool,
    default: String,
    default_pull_raw: String,
}

impl ParsedValueBody {
    fn parse(body_raw: &str) -> Option<Self> {
        let locked = text::extract_tag_text(body_raw, "locked")?.trim() == "1";
        let locked_default_current =
            text::extract_tag_text(body_raw, "lockedDefaultCurrent")?.trim() == "1";
        let default = text::decode(text::extract_tag_text(body_raw, "default")?);
        let default_pull_raw = text::extract_tag_text(body_raw, "defaultPull")?
            .trim()
            .to_string();
        Some(ParsedValueBody {
            locked,
            locked_default_current,
            default,
            default_pull_raw,
        })
    }

    fn to_raw(&self) -> String {
        format!(
            "<locked>{}</locked><lockedDefaultCurrent>{}</lockedDefaultCurrent><default>{}</default><defaultPull>{}</defaultPull>",
            if self.locked { "1" } else { "0" },
            if self.locked_default_current { "1" } else { "0" },
            text::encode_cdata(&self.default),
            self.default_pull_raw,
        )
    }

    fn to_yaml(&self) -> Value {
        let mut m = Mapping::new();
        m.insert("locked".into(), Value::Bool(self.locked));
        m.insert(
            "lockedDefaultCurrent".into(),
            Value::Bool(self.locked_default_current),
        );
        m.insert("default".into(), Value::String(self.default.clone()));
        m.insert(
            "defaultPull".into(),
            Value::String(self.default_pull_raw.clone()),
        );
        Value::Mapping(m)
    }

    fn from_yaml(v: &Value) -> Option<Self> {
        let m = v.as_mapping()?;
        let locked = m.get(Value::String("locked".into()))?.as_bool()?;
        let locked_default_current = m
            .get(Value::String("lockedDefaultCurrent".into()))?
            .as_bool()?;
        let default = m
            .get(Value::String("default".into()))?
            .as_str()?
            .to_string();
        let pull_value = m.get(Value::String("defaultPull".into()))?;
        let default_pull_raw = match pull_value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => if *b { "1" } else { "0" }.to_string(),
            _ => return None,
        };
        Some(ParsedValueBody {
            locked,
            locked_default_current,
            default,
            default_pull_raw,
        })
    }
}

fn encode_value_body(v: &ValueEntry) -> Value {
    if let Some(parsed) = ParsedValueBody::parse(&v.body_raw) {
        if parsed.to_raw() == v.body_raw {
            return parsed.to_yaml();
        }
    }
    raw_xml_value(&v.body_raw)
}

/// A `<midi>`/`<osc>` message binding. Represented as raw XML text for
/// now — see docs/FORMAT.md for the internal shape; a future typed
/// builder layer can upgrade this the same verified-reconstruction way
/// properties and values are handled, without changing this shape for
/// bindings it still doesn't understand.
#[derive(Serialize, Deserialize)]
pub struct TextMessage {
    pub kind: String,
    pub raw_xml: String,
}

impl TextMessage {
    fn from_message(m: &Message) -> Self {
        TextMessage {
            kind: m.kind.to_string(),
            raw_xml: m.raw.clone(),
        }
    }

    fn into_message(self) -> Result<Message> {
        let kind = match self.kind.as_str() {
            "midi" => MessageKind::Midi,
            "osc" => MessageKind::Osc,
            other => return Err(Error::Malformed(format!("unknown message kind '{other}'"))),
        };
        Ok(Message {
            kind,
            raw: self.raw_xml,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{Node, Property};

    #[test]
    fn dump_build_round_trips_a_simple_layout() {
        let mut root = Node::new("GROUP");
        root.id = "11111111-1111-1111-1111-111111111111".to_string();
        root.properties.push(Property::bool("background", true));
        root.properties.push(Property::float("cornerRadius", 1.0));
        root.properties
            .push(Property::color("color", Color::new(0.0, 0.25, 1.0, 1.0)));
        root.values
            .push(ValueEntry::new("touch", "<![CDATA[false]]>"));
        let layout = Layout {
            lexml_version: "5".to_string(),
            root,
        };

        let yaml = dump(&layout).unwrap();
        assert!(yaml.contains("cornerRadius"));
        let rebuilt = build(&yaml).unwrap();
        assert_eq!(layout, rebuilt);
    }

    #[test]
    fn unknown_type_code_falls_back_to_raw_xml_in_yaml() {
        let mut root = Node::new("GROUP");
        root.properties.push(Property {
            type_code: "x".into(),
            key_raw: "<![CDATA[future]]>".into(),
            value_raw: "<weird><nested/></weird>".into(),
        });
        let layout = Layout {
            lexml_version: "5".into(),
            root,
        };
        let yaml = dump(&layout).unwrap();
        assert!(yaml.contains("raw_xml"));
        let rebuilt = build(&yaml).unwrap();
        assert_eq!(layout, rebuilt);
    }

    #[test]
    fn script_property_can_reference_an_external_lua_file() {
        let dir = std::env::temp_dir().join(format!("tosc-yaml-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let script_path = dir.join("on_init.lua");
        std::fs::write(&script_path, "function init()\n  print('hi')\nend\n").unwrap();

        let yaml = r#"
lexml_version: '5'
root:
  id: 11111111-1111-1111-1111-111111111111
  type: BUTTON
  properties:
  - type: s
    key: script
    value:
      file: on_init.lua
  values: []
"#;
        let layout = build_with_base(yaml, &dir).unwrap();
        let script = layout.root.property("script").unwrap();
        assert_eq!(
            script.as_string().unwrap(),
            "function init()\n  print('hi')\nend\n"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn file_reference_on_a_non_string_property_is_an_error() {
        let yaml = r#"
lexml_version: '5'
root:
  id: 11111111-1111-1111-1111-111111111111
  type: BUTTON
  properties:
  - type: i
    key: shape
    value:
      file: whatever.lua
  values: []
"#;
        assert!(build(yaml).is_err());
    }
}
