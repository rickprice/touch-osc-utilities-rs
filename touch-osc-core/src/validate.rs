//! Validation for a parsed [`Layout`]: catches structural/content
//! mistakes before the file goes anywhere near the real editor.
//!
//! Every check here is derived either from an invariant confirmed across
//! every fixture in `tests/fixtures/` (see docs/FORMAT.md) or from a
//! basic OSC/MIDI protocol rule (reserved address characters, the MIDI
//! channel range) — not a guessed rule about the format. Checks are
//! split into [`Severity::Error`] (something the format/protocol itself
//! rules out) and [`Severity::Warning`] (never observed in a fixture,
//! likely a mistake, but not provably invalid).

use crate::text;
use crate::tree::{Layout, Message, MessageKind, Node};
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub severity: Severity,
    pub node_id: Option<String>,
    pub message: String,
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sev = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        match &self.node_id {
            Some(id) => write!(f, "{sev}: [{id}] {}", self.message),
            None => write!(f, "{sev}: {}", self.message),
        }
    }
}

/// Run every check against `layout`, in no particular severity order.
pub fn validate(layout: &Layout) -> Vec<Issue> {
    let mut issues = Vec::new();
    check_duplicate_ids(layout, &mut issues);
    check_node(&layout.root, None, &mut issues);
    issues
}

pub fn has_errors(issues: &[Issue]) -> bool {
    issues.iter().any(|i| i.severity == Severity::Error)
}

fn check_duplicate_ids(layout: &Layout, issues: &mut Vec<Issue>) {
    let mut counts: HashMap<&str, u32> = HashMap::new();
    for node in layout.root.walk() {
        *counts.entry(node.id.as_str()).or_insert(0) += 1;
    }
    let mut reported = HashSet::new();
    for node in layout.root.walk() {
        let count = counts[node.id.as_str()];
        if count > 1 && reported.insert(node.id.clone()) {
            issues.push(Issue {
                severity: Severity::Error,
                node_id: Some(node.id.clone()),
                message: format!("duplicate node ID, shared by {count} nodes"),
            });
        }
    }
}

const KNOWN_TYPES: &[&str] = &[
    "GROUP", "PAGER", "BOX", "BUTTON", "LABEL", "TEXT", "FADER", "XY", "RADIAL", "ENCODER",
    "RADAR", "RADIO", "GRID",
];

/// `GRID` holds per-cell template children, in addition to the
/// GROUP/PAGER container nesting -- see docs/FORMAT.md.
const CONTAINER_TYPES: &[&str] = &["GROUP", "PAGER", "GRID"];

/// `(key, expected type code)` for properties that had exactly one type
/// code across every fixture. `gridX`/`gridY` are deliberately excluded:
/// they are `b` (snap-to-grid toggle) on `XY`/`RADAR` but `i` (column/row
/// count) on `GRID` -- see docs/FORMAT.md's documented hazard.
const PROPERTY_TYPES: &[(&str, &str)] = &[
    ("background", "b"),
    ("bar", "b"),
    ("barDisplay", "i"),
    ("buttonType", "i"),
    ("centered", "b"),
    ("color", "c"),
    ("cornerRadius", "f"),
    ("cursor", "b"),
    ("cursorDisplay", "i"),
    ("exclusive", "b"),
    ("font", "i"),
    ("frame", "r"),
    ("grabFocus", "b"),
    ("grid", "b"),
    ("gridColor", "c"),
    ("gridNaming", "i"),
    ("gridOrder", "i"),
    ("gridStart", "i"),
    ("gridSteps", "i"),
    ("gridStepsX", "i"),
    ("gridStepsY", "i"),
    ("gridType", "i"),
    ("interactive", "b"),
    ("inverted", "b"),
    ("lines", "b"),
    ("linesDisplay", "i"),
    ("lockX", "b"),
    ("lockY", "b"),
    ("locked", "b"),
    ("name", "s"),
    ("orientation", "i"),
    ("outline", "b"),
    ("outlineStyle", "i"),
    ("pointerPriority", "i"),
    ("press", "b"),
    ("radioType", "i"),
    ("release", "b"),
    ("response", "i"),
    ("responseFactor", "i"),
    ("script", "s"),
    ("shape", "i"),
    ("steps", "i"),
    ("tabColorOff", "c"),
    ("tabColorOn", "c"),
    ("tabLabel", "s"),
    ("tabLabels", "b"),
    ("tabbar", "b"),
    ("tabbarDoubleTap", "b"),
    ("tabbarSize", "i"),
    ("tag", "s"),
    ("textAlignH", "i"),
    ("textAlignV", "i"),
    ("textClip", "b"),
    ("textColor", "c"),
    ("textColorOff", "c"),
    ("textColorOn", "c"),
    ("textLength", "i"),
    ("textSize", "i"),
    ("textSizeOff", "i"),
    ("textSizeOn", "i"),
    ("textWrap", "b"),
    ("valuePosition", "b"),
    ("visible", "b"),
];

fn expected_type(key: &str) -> Option<&'static str> {
    PROPERTY_TYPES
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, t)| *t)
}

const OSC_ILLEGAL_CHARS: &[char] = &[' ', '#', '*', ',', '?', '[', ']', '{', '}'];

fn illegal_osc_chars(s: &str) -> Option<&'static str> {
    if s.is_empty() {
        return Some("is empty");
    }
    if s.chars().any(|c| OSC_ILLEGAL_CHARS.contains(&c)) {
        return Some(
            "contains a character reserved in OSC address patterns (space # * , ? [ ] { })",
        );
    }
    None
}

fn check_node(node: &Node, parent_type: Option<&str>, issues: &mut Vec<Issue>) {
    if !KNOWN_TYPES.contains(&node.node_type.as_str()) {
        issues.push(Issue {
            severity: Severity::Warning,
            node_id: Some(node.id.clone()),
            message: format!(
                "unrecognized node type '{}' (not seen in any fixture; may be fine, just unverified)",
                node.node_type
            ),
        });
    }

    if !node.children.is_empty() && !CONTAINER_TYPES.contains(&node.node_type.as_str()) {
        issues.push(Issue {
            severity: Severity::Warning,
            node_id: Some(node.id.clone()),
            message: format!(
                "node type '{}' has children, but only GROUP/PAGER/GRID were observed with children in the fixtures",
                node.node_type
            ),
        });
    }

    if parent_type == Some("PAGER") && node.node_type != "GROUP" {
        issues.push(Issue {
            severity: Severity::Warning,
            node_id: Some(node.id.clone()),
            message: format!(
                "PAGER child has type '{}', but every PAGER child observed in the fixtures was a GROUP",
                node.node_type
            ),
        });
    }

    match node.property("frame") {
        None => issues.push(Issue {
            severity: Severity::Error,
            node_id: Some(node.id.clone()),
            message: "missing required 'frame' property (present on every node in every fixture)"
                .into(),
        }),
        Some(p) if p.as_rect().is_none() => issues.push(Issue {
            severity: Severity::Error,
            node_id: Some(node.id.clone()),
            message: "'frame' property does not parse as a rect".into(),
        }),
        _ => {}
    }

    if node.value("touch").is_none() {
        issues.push(Issue {
            severity: Severity::Warning,
            node_id: Some(node.id.clone()),
            message: "missing 'touch' value entry (present on every node in every fixture)".into(),
        });
    }

    match node.property("name").and_then(|p| p.as_string()) {
        None => issues.push(Issue {
            severity: Severity::Warning,
            node_id: Some(node.id.clone()),
            message: "missing 'name' property; an OSC message using the PROPERTY 'name'/'parent.name' partial would address an empty segment".into(),
        }),
        Some(name) => {
            if let Some(reason) = illegal_osc_chars(&name) {
                issues.push(Issue {
                    severity: Severity::Error,
                    node_id: Some(node.id.clone()),
                    message: format!("name '{name}' is not a valid OSC address segment: {reason}"),
                });
            }
        }
    }

    for prop in &node.properties {
        let Some(expected) = expected_type(&prop.key()) else {
            continue;
        };
        if prop.type_code != expected {
            issues.push(Issue {
                severity: Severity::Error,
                node_id: Some(node.id.clone()),
                message: format!(
                    "property '{}' has type '{}', expected '{}'",
                    prop.key(),
                    prop.type_code,
                    expected
                ),
            });
            continue;
        }
        let parses = match expected {
            "b" => prop.as_bool().is_some(),
            "i" => prop.as_int().is_some(),
            "f" => prop.as_float().is_some(),
            "c" => prop.as_color().is_some(),
            "r" => prop.as_rect().is_some(),
            _ => true,
        };
        if !parses {
            issues.push(Issue {
                severity: Severity::Error,
                node_id: Some(node.id.clone()),
                message: format!(
                    "property '{}' is declared type '{}' but its value doesn't parse as one",
                    prop.key(),
                    expected
                ),
            });
        }
    }

    check_lua_syntax(node, issues);

    for message in &node.messages {
        check_message(node, message, issues);
    }

    let mut seen_names = HashSet::new();
    for child in &node.children {
        if let Some(name) = child.property("name").and_then(|p| p.as_string()) {
            if !seen_names.insert(name.clone()) {
                issues.push(Issue {
                    severity: Severity::Warning,
                    node_id: Some(child.id.clone()),
                    message: format!(
                        "sibling control also named '{name}'; OSC addresses built from 'name' won't be unique"
                    ),
                });
            }
        }
        check_node(child, Some(node.node_type.as_str()), issues);
    }
}

/// Parse the `script` property's Lua source, if present, and report any
/// syntax error. This only checks syntax (via `full_moon`, a lossless
/// Lua parser) -- it can't catch a script that's syntactically valid but
/// wrong, but a syntax error is exactly the kind of mistake that's silent
/// until you open the file in the editor and the control just doesn't
/// work. Only the `script` key is checked (not every `s`-type property)
/// since that's the only key observed holding Lua source -- see
/// docs/FORMAT.md.
fn check_lua_syntax(node: &Node, issues: &mut Vec<Issue>) {
    let Some(prop) = node.property("script") else {
        return;
    };
    let Some(code) = prop.as_string() else {
        return;
    };
    if let Err(errors) = full_moon::parse(&code) {
        let detail = errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        issues.push(Issue {
            severity: Severity::Error,
            node_id: Some(node.id.clone()),
            message: format!("'script' has a Lua syntax error: {detail}"),
        });
    }
}

fn check_message(node: &Node, message: &Message, issues: &mut Vec<Issue>) {
    match message.kind {
        MessageKind::Osc => match text::extract_tag_text(&message.raw, "path") {
            None => issues.push(Issue {
                severity: Severity::Error,
                node_id: Some(node.id.clone()),
                message: "OSC message has no <path>".into(),
            }),
            Some(path) => {
                for body in find_all_tag_bodies(path, "partial") {
                    let Some(ptype) = text::extract_tag_text(body, "type") else {
                        continue;
                    };
                    if ptype != "CONSTANT" {
                        continue;
                    }
                    let Some(raw_value) = text::extract_tag_text(body, "value") else {
                        continue;
                    };
                    let value = text::decode(raw_value);
                    if value == "/" {
                        continue; // the path separator literal, not a segment
                    }
                    if let Some(reason) = illegal_osc_chars(&value) {
                        issues.push(Issue {
                            severity: Severity::Error,
                            node_id: Some(node.id.clone()),
                            message: format!("OSC path constant '{value}' is invalid: {reason}"),
                        });
                    }
                }
            }
        },
        MessageKind::Midi => {
            if let Some(channel_text) = text::extract_tag_text(&message.raw, "channel") {
                if let Ok(channel) = channel_text.trim().parse::<i64>() {
                    if !(0..=15).contains(&channel) {
                        issues.push(Issue {
                            severity: Severity::Error,
                            node_id: Some(node.id.clone()),
                            message: format!("MIDI channel {channel} out of range 0-15"),
                        });
                    }
                }
            }
        }
    }
}

/// Split `s` into the bodies of each top-level, non-nested `<tag>...</tag>`
/// occurrence (used for repeated sibling elements like `<partial>`).
fn find_all_tag_bodies<'a>(s: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(start) = rest.find(open.as_str()) {
        let after_open = &rest[start + open.len()..];
        match after_open.find(close.as_str()) {
            Some(end) => {
                out.push(&after_open[..end]);
                rest = &after_open[end + close.len()..];
            }
            None => break,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls;
    use crate::messages::{osc_binding, Conversion};
    use crate::tree::Node;

    fn layout_from(root: Node) -> Layout {
        Layout {
            lexml_version: "5".into(),
            root,
        }
    }

    #[test]
    fn clean_builder_output_has_no_errors() {
        let node: Node = controls::button("play").build();
        let issues = validate(&layout_from(node));
        assert!(!has_errors(&issues), "{issues:?}");
    }

    #[test]
    fn detects_duplicate_ids() {
        let mut root: Node = controls::group("root").build();
        let mut child = controls::button("a").build();
        child.id = root.id.clone();
        root.children.push(child);
        let issues = validate(&layout_from(root));
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("duplicate node ID")));
    }

    #[test]
    fn detects_missing_frame() {
        let mut node: Node = controls::button("play").build();
        node.properties.retain(|p| p.key() != "frame");
        let issues = validate(&layout_from(node));
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("frame")));
    }

    #[test]
    fn detects_bad_property_type() {
        let mut node: Node = controls::button("play").build();
        node.set_property(crate::tree::Property::int("background", 1));
        let issues = validate(&layout_from(node));
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("background")));
    }

    #[test]
    fn detects_invalid_osc_name() {
        let node: Node = controls::button("bad name").build();
        let issues = validate(&layout_from(node));
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("not a valid OSC")));
    }

    #[test]
    fn detects_invalid_nesting() {
        let node: Node = controls::button("a").child(controls::button("b")).build();
        let issues = validate(&layout_from(node));
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Warning && i.message.contains("has children")));
    }

    #[test]
    fn detects_bad_osc_path_constant() {
        let node: Node = controls::button("play")
            .message(osc_binding("x", Conversion::Float, false))
            .build();
        // Corrupt the constant "/" into something illegal.
        let mut node = node;
        node.messages[0].raw = node.messages[0].raw.replace(
            "<value><![CDATA[/]]></value>",
            "<value><![CDATA[/bad path]]></value>",
        );
        let issues = validate(&layout_from(node));
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("OSC path constant")));
    }

    #[test]
    fn detects_duplicate_sibling_names() {
        let node: Node = controls::group("root")
            .child(controls::button("dup"))
            .child(controls::button("dup"))
            .build();
        let issues = validate(&layout_from(node));
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Warning && i.message.contains("also named")));
    }

    #[test]
    fn detects_lua_syntax_error_in_script() {
        let mut node: Node = controls::button("play").build();
        node.set_property(crate::tree::Property::string(
            "script",
            "function init(\n  print('missing closing parens'\nend",
        ));
        let issues = validate(&layout_from(node));
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("Lua syntax error")));
    }

    #[test]
    fn valid_lua_script_has_no_errors() {
        let mut node: Node = controls::button("play").build();
        node.set_property(crate::tree::Property::string(
            "script",
            "function init()\n  self.color = Color(1, 0, 0, 1)\nend\n",
        ));
        let issues = validate(&layout_from(node));
        assert!(!issues
            .iter()
            .any(|i| i.message.contains("Lua syntax error")));
    }
}
