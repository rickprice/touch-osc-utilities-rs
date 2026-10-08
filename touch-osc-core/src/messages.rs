//! Typed OSC/MIDI message binding builders.
//!
//! These build the exact shapes observed in `tests/fixtures/` (see
//! `docs/FORMAT.md`'s "`<messages>`" section) rather than a guessed
//! schema. Every OSC binding found in the fixtures builds its path from
//! `CONSTANT`/`PROPERTY` partials — either `/<name>` (a control directly
//! under the layout root/pager) or `/<parent.name>/<name>` (a control
//! nested one level deeper, inside a page or group) — with a single
//! `VALUE` argument tied back to the control's own value key (`x`, `y`,
//! `page`, `text`, ...). Every MIDI binding is a `CONTROLCHANGE` message
//! with the same `CONSTANT`/`INDEX`/`VALUE` triple.
//!
//! For anything else — a shape not in the fixtures, a different MIDI
//! message type — construct a [`Message`] directly with a hand-written
//! `raw` XML fragment; nothing requires going through this module.

use crate::tree::{Message, MessageKind};

/// The `conversion` applied to an OSC path/argument partial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conversion {
    String,
    Integer,
    Float,
    Boolean,
}

impl Conversion {
    fn as_str(self) -> &'static str {
        match self {
            Conversion::String => "STRING",
            Conversion::Integer => "INTEGER",
            Conversion::Float => "FLOAT",
            Conversion::Boolean => "BOOLEAN",
        }
    }
}

const CONNECTIONS_ALL: &str = "1111111111";

/// The standard OSC binding observed on every scripted/bound control in
/// the fixtures: send+receive enabled on all connections, triggered by
/// any change to `value_key`, with a single argument carrying that same
/// value. Set `nested` when the control is one level deeper than its
/// addressed context (its path becomes `/<parent.name>/<name>` instead
/// of `/<name>`) — see docs/FORMAT.md.
pub fn osc_binding(value_key: &str, conversion: Conversion, nested: bool) -> Message {
    let parent_partial = if nested {
        concat!(
            "<partial><type>PROPERTY</type><conversion>STRING</conversion>",
            "<value><![CDATA[parent.name]]></value><scaleMin>0</scaleMin><scaleMax>1</scaleMax></partial>",
            "<partial><type>CONSTANT</type><conversion>STRING</conversion>",
            "<value><![CDATA[/]]></value><scaleMin>0</scaleMin><scaleMax>1</scaleMax></partial>",
        )
    } else {
        ""
    };
    let raw = format!(
        concat!(
            "<enabled>1</enabled><send>1</send><receive>1</receive><feedback>0</feedback>",
            "<noDuplicates>0</noDuplicates><connections>{connections}</connections>",
            "<triggers><trigger><var><![CDATA[{key}]]></var><condition>ANY</condition></trigger></triggers>",
            "<path>",
            "<partial><type>CONSTANT</type><conversion>STRING</conversion>",
            "<value><![CDATA[/]]></value><scaleMin>0</scaleMin><scaleMax>1</scaleMax></partial>",
            "{parent_partial}",
            "<partial><type>PROPERTY</type><conversion>STRING</conversion>",
            "<value><![CDATA[name]]></value><scaleMin>0</scaleMin><scaleMax>1</scaleMax></partial>",
            "</path>",
            "<arguments>",
            "<partial><type>VALUE</type><conversion>{conversion}</conversion>",
            "<value><![CDATA[{key}]]></value><scaleMin>0</scaleMin><scaleMax>1</scaleMax></partial>",
            "</arguments>",
        ),
        connections = CONNECTIONS_ALL,
        key = value_key,
        parent_partial = parent_partial,
        conversion = conversion.as_str(),
    );
    Message {
        kind: MessageKind::Osc,
        raw,
    }
}

/// The standard MIDI CC binding observed in the fixtures: a
/// `CONTROLCHANGE` message on `channel` (0-15), triggered by any change
/// to `value_key`, scaled to `scale_max` (`127` for a continuous
/// slider-style value, `1` for a discrete/boolean-style one — see the
/// values observed for `x`/`y` vs. `page` in docs/FORMAT.md).
pub fn midi_cc_binding(value_key: &str, channel: u8, scale_max: f64) -> Message {
    let raw = format!(
        concat!(
            "<enabled>1</enabled><send>1</send><receive>1</receive><feedback>0</feedback>",
            "<noDuplicates>0</noDuplicates><connections>{connections}</connections>",
            "<triggers><trigger><var><![CDATA[{key}]]></var><condition>ANY</condition></trigger></triggers>",
            "<message><type>CONTROLCHANGE</type><channel>{channel}</channel><data1>0</data1><data2>0</data2></message>",
            "<values>",
            "<value><type>CONSTANT</type><key/><scaleMin>0</scaleMin><scaleMax>15</scaleMax></value>",
            "<value><type>INDEX</type><key/><scaleMin>0</scaleMin><scaleMax>1</scaleMax></value>",
            "<value><type>VALUE</type><key><![CDATA[{key}]]></key><scaleMin>0</scaleMin><scaleMax>{scale_max}</scaleMax></value>",
            "</values>",
        ),
        connections = CONNECTIONS_ALL,
        key = value_key,
        channel = channel,
        scale_max = crate::text::format_float(scale_max),
    );
    Message {
        kind: MessageKind::Midi,
        raw,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls;
    use crate::tree::{Layout, Node};
    use crate::xml;

    #[test]
    fn osc_binding_round_trips_inside_a_layout() {
        let node: Node = controls::fader("level")
            .message(osc_binding("x", Conversion::Float, true))
            .build();
        let layout = Layout {
            lexml_version: "5".into(),
            root: node,
        };
        let xml_str = xml::serialize_layout(&layout);
        let reparsed = xml::parse_layout(&xml_str).unwrap();
        assert_eq!(layout, reparsed);
        assert_eq!(reparsed.root.messages[0].kind, MessageKind::Osc);
        assert!(reparsed.root.messages[0].raw.contains("parent.name"));
    }

    #[test]
    fn midi_binding_round_trips_inside_a_layout() {
        let node: Node = controls::fader("level")
            .message(midi_cc_binding("x", 0, 127.0))
            .build();
        let layout = Layout {
            lexml_version: "5".into(),
            root: node,
        };
        let xml_str = xml::serialize_layout(&layout);
        let reparsed = xml::parse_layout(&xml_str).unwrap();
        assert_eq!(layout, reparsed);
        assert!(reparsed.root.messages[0].raw.contains("CONTROLCHANGE"));
        assert!(reparsed.root.messages[0]
            .raw
            .contains("<scaleMax>127</scaleMax>"));
    }
}
