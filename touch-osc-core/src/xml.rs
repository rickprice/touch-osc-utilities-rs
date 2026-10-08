//! XML parsing and serialization for the decompressed `.tosc` document.
//!
//! The parser slices raw byte ranges directly out of the source string for
//! anything that becomes a `*_raw` field on [`crate::tree`] types, rather
//! than reconstructing text from decoded XML events. That is what makes
//! CDATA sections, numeral formatting, and unrecognized sub-structure
//! (future property/message shapes) survive byte-for-byte: the parser
//! never needs to understand a shape in order to preserve it, it only
//! needs to find where it starts and ends.

use crate::error::{Error, Result};
use crate::tree::{Layout, Message, MessageKind, Node, Property, ValueEntry};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;

pub fn parse_layout(src: &str) -> Result<Layout> {
    let mut reader = Reader::from_str(src);
    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"lexml" => {
                let version = get_attr(&e, b"version")?.unwrap_or_default();
                let root = loop {
                    match reader.read_event()? {
                        Event::Start(ne) if ne.name().as_ref() == b"node" => {
                            break parse_node_from_start(&mut reader, src, &ne)?;
                        }
                        Event::Eof => return Err(Error::Malformed("<lexml> has no <node>".into())),
                        _ => {}
                    }
                };
                expect_end(&mut reader, b"lexml")?;
                return Ok(Layout {
                    lexml_version: version,
                    root,
                });
            }
            Event::Eof => return Err(Error::Malformed("missing <lexml> root element".into())),
            _ => {}
        }
    }
}

fn parse_node_from_start(
    reader: &mut Reader<&[u8]>,
    src: &str,
    start: &BytesStart,
) -> Result<Node> {
    let id = get_attr(start, b"ID")?
        .ok_or_else(|| Error::Malformed("<node> missing ID attribute".into()))?;
    let node_type = get_attr(start, b"type")?
        .ok_or_else(|| Error::Malformed("<node> missing type attribute".into()))?;
    let mut node = Node {
        id,
        node_type,
        properties: Vec::new(),
        values: Vec::new(),
        messages: Vec::new(),
        children: Vec::new(),
    };
    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"properties" => {
                node.properties = parse_properties(reader, src)?;
            }
            Event::Start(e) if e.name().as_ref() == b"values" => {
                node.values = parse_values(reader, src)?;
            }
            Event::Start(e) if e.name().as_ref() == b"messages" => {
                node.messages = parse_messages(reader, src)?;
            }
            Event::Start(e) if e.name().as_ref() == b"children" => {
                node.children = parse_children(reader, src)?;
            }
            Event::End(e) if e.name().as_ref() == b"node" => break,
            Event::Eof => return Err(Error::Malformed("unexpected EOF inside <node>".into())),
            _ => {}
        }
    }
    Ok(node)
}

fn parse_children(reader: &mut Reader<&[u8]>, src: &str) -> Result<Vec<Node>> {
    let mut out = Vec::new();
    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"node" => {
                out.push(parse_node_from_start(reader, src, &e)?);
            }
            Event::End(e) if e.name().as_ref() == b"children" => break,
            Event::Eof => return Err(Error::Malformed("unexpected EOF inside <children>".into())),
            _ => {}
        }
    }
    Ok(out)
}

fn parse_properties(reader: &mut Reader<&[u8]>, src: &str) -> Result<Vec<Property>> {
    let mut out = Vec::new();
    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"property" => {
                let type_code = get_attr(&e, b"type")?.unwrap_or_default();
                let mut key_raw = String::new();
                let mut value_raw = String::new();
                loop {
                    match reader.read_event()? {
                        Event::Start(ke) if ke.name().as_ref() == b"key" => {
                            key_raw = capture_raw_inner(reader, src)?;
                        }
                        Event::Empty(ke) if ke.name().as_ref() == b"key" => {}
                        Event::Start(ve) if ve.name().as_ref() == b"value" => {
                            value_raw = capture_raw_inner(reader, src)?;
                        }
                        Event::Empty(ve) if ve.name().as_ref() == b"value" => {}
                        Event::End(ee) if ee.name().as_ref() == b"property" => break,
                        Event::Eof => {
                            return Err(Error::Malformed("unexpected EOF inside <property>".into()))
                        }
                        _ => {}
                    }
                }
                out.push(Property {
                    type_code,
                    key_raw,
                    value_raw,
                });
            }
            Event::End(e) if e.name().as_ref() == b"properties" => break,
            Event::Eof => {
                return Err(Error::Malformed(
                    "unexpected EOF inside <properties>".into(),
                ))
            }
            _ => {}
        }
    }
    Ok(out)
}

fn parse_values(reader: &mut Reader<&[u8]>, src: &str) -> Result<Vec<ValueEntry>> {
    let mut out = Vec::new();
    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"value" => {
                let mut key_raw = String::new();
                loop {
                    match reader.read_event()? {
                        Event::Start(ke) if ke.name().as_ref() == b"key" => {
                            key_raw = capture_raw_inner(reader, src)?;
                            break;
                        }
                        Event::Empty(ke) if ke.name().as_ref() == b"key" => break,
                        Event::Eof => {
                            return Err(Error::Malformed("unexpected EOF inside <value>".into()))
                        }
                        _ => {}
                    }
                }
                let body_raw = capture_raw_inner_until(reader, src, b"value")?;
                out.push(ValueEntry { key_raw, body_raw });
            }
            Event::Empty(e) if e.name().as_ref() == b"value" => {
                out.push(ValueEntry {
                    key_raw: String::new(),
                    body_raw: String::new(),
                });
            }
            Event::End(e) if e.name().as_ref() == b"values" => break,
            Event::Eof => return Err(Error::Malformed("unexpected EOF inside <values>".into())),
            _ => {}
        }
    }
    Ok(out)
}

fn parse_messages(reader: &mut Reader<&[u8]>, src: &str) -> Result<Vec<Message>> {
    let mut out = Vec::new();
    loop {
        match reader.read_event()? {
            Event::Start(e) if e.name().as_ref() == b"midi" => {
                let raw = capture_raw_inner(reader, src)?;
                out.push(Message {
                    kind: MessageKind::Midi,
                    raw,
                });
            }
            Event::Start(e) if e.name().as_ref() == b"osc" => {
                let raw = capture_raw_inner(reader, src)?;
                out.push(Message {
                    kind: MessageKind::Osc,
                    raw,
                });
            }
            Event::End(e) if e.name().as_ref() == b"messages" => break,
            Event::Eof => return Err(Error::Malformed("unexpected EOF inside <messages>".into())),
            _ => {}
        }
    }
    Ok(out)
}

/// Capture the raw source text between the just-consumed `Start` event and
/// its matching `End`, tracking generic element depth so nested elements
/// (of any name) don't confuse the match.
fn capture_raw_inner(reader: &mut Reader<&[u8]>, src: &str) -> Result<String> {
    let start_pos = reader.buffer_position() as usize;
    capture_raw_inner_from(reader, src, start_pos)
}

/// Like [`capture_raw_inner`], but for a start tag that was consumed
/// earlier (e.g. a `<key>` sibling was read first); captures from the
/// reader's current position to the matching end of `end_name`.
fn capture_raw_inner_until(
    reader: &mut Reader<&[u8]>,
    src: &str,
    end_name: &[u8],
) -> Result<String> {
    let start_pos = reader.buffer_position() as usize;
    let _ = end_name;
    capture_raw_inner_from(reader, src, start_pos)
}

fn capture_raw_inner_from(
    reader: &mut Reader<&[u8]>,
    src: &str,
    start_pos: usize,
) -> Result<String> {
    let mut depth: i32 = 0;
    loop {
        match reader.read_event()? {
            Event::Start(_) => depth += 1,
            Event::End(e) => {
                if depth == 0 {
                    let end_pos_after = reader.buffer_position() as usize;
                    let close_len = 3 + e.name().as_ref().len();
                    let end_pos_before = end_pos_after - close_len;
                    return Ok(src[start_pos..end_pos_before].to_string());
                }
                depth -= 1;
            }
            Event::Eof => return Err(Error::Malformed("unexpected EOF capturing raw text".into())),
            _ => {}
        }
    }
}

fn expect_end(reader: &mut Reader<&[u8]>, name: &[u8]) -> Result<()> {
    loop {
        match reader.read_event()? {
            Event::End(e) if e.name().as_ref() == name => return Ok(()),
            Event::Eof => {
                return Err(Error::Malformed(format!(
                    "expected </{}>",
                    String::from_utf8_lossy(name)
                )))
            }
            _ => {}
        }
    }
}

fn get_attr(e: &BytesStart, name: &[u8]) -> Result<Option<String>> {
    for attr in e.attributes() {
        let attr = attr?;
        if attr.key.as_ref() == name {
            let value = attr.unescape_value()?;
            return Ok(Some(value.into_owned()));
        }
    }
    Ok(None)
}

// --- serialization -------------------------------------------------------

pub fn serialize_layout(layout: &Layout) -> String {
    let mut out = String::new();
    out.push_str("<?xml version='1.0' encoding='UTF-8'?>");
    out.push_str("<lexml version='");
    escape_attr_into(&layout.lexml_version, &mut out);
    out.push_str("'>");
    serialize_node(&layout.root, &mut out);
    out.push_str("</lexml>");
    out
}

fn serialize_node(node: &Node, out: &mut String) {
    out.push_str("<node ID='");
    escape_attr_into(&node.id, out);
    out.push_str("' type='");
    escape_attr_into(&node.node_type, out);
    out.push_str("'>");

    out.push_str("<properties>");
    for p in &node.properties {
        out.push_str("<property type='");
        escape_attr_into(&p.type_code, out);
        out.push_str("'><key>");
        out.push_str(&p.key_raw);
        out.push_str("</key><value>");
        out.push_str(&p.value_raw);
        out.push_str("</value></property>");
    }
    out.push_str("</properties>");

    out.push_str("<values>");
    for v in &node.values {
        out.push_str("<value><key>");
        out.push_str(&v.key_raw);
        out.push_str("</key>");
        out.push_str(&v.body_raw);
        out.push_str("</value>");
    }
    out.push_str("</values>");

    if !node.messages.is_empty() {
        out.push_str("<messages>");
        for m in &node.messages {
            let tag = m.kind.to_string();
            out.push('<');
            out.push_str(&tag);
            out.push('>');
            out.push_str(&m.raw);
            out.push_str("</");
            out.push_str(&tag);
            out.push('>');
        }
        out.push_str("</messages>");
    }

    if !node.children.is_empty() {
        out.push_str("<children>");
        for c in &node.children {
            serialize_node(c, out);
        }
        out.push_str("</children>");
    }

    out.push_str("</node>");
}

fn escape_attr_into(s: &str, out: &mut String) {
    out.push_str(&quick_xml::escape::escape(s));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::Color;

    #[test]
    fn parses_and_reserializes_minimal_doc() {
        let xml = "<?xml version='1.0' encoding='UTF-8'?><lexml version='5'><node ID='abc' type='GROUP'><properties><property type='b'><key><![CDATA[background]]></key><value>1</value></property></properties><values><value><key><![CDATA[touch]]></key><locked>0</locked><lockedDefaultCurrent>0</lockedDefaultCurrent><default><![CDATA[false]]></default><defaultPull>0</defaultPull></value></values></node></lexml>";
        let layout = parse_layout(xml).unwrap();
        assert_eq!(layout.lexml_version, "5");
        assert_eq!(layout.root.id, "abc");
        assert_eq!(layout.root.node_type, "GROUP");
        assert_eq!(layout.root.properties.len(), 1);
        assert_eq!(layout.root.properties[0].key(), "background");
        assert_eq!(layout.root.properties[0].as_bool(), Some(true));
        assert_eq!(layout.root.values[0].key(), "touch");

        let reserialized = serialize_layout(&layout);
        let reparsed = parse_layout(&reserialized).unwrap();
        assert_eq!(layout, reparsed);
    }

    #[test]
    fn preserves_color_property_raw_text() {
        let xml = "<?xml version='1.0' encoding='UTF-8'?><lexml version='5'><node ID='x' type='GROUP'><properties><property type='c'><key><![CDATA[color]]></key><value><r>0</r><g>0.25</g><b>1</b><a>1</a></value></property></properties><values></values></node></lexml>";
        let layout = parse_layout(xml).unwrap();
        let p = &layout.root.properties[0];
        assert_eq!(p.value_raw, "<r>0</r><g>0.25</g><b>1</b><a>1</a>");
        assert_eq!(p.as_color(), Some(Color::new(0.0, 0.25, 1.0, 1.0)));
    }

    #[test]
    fn preserves_messages_raw() {
        let xml = "<?xml version='1.0' encoding='UTF-8'?><lexml version='5'><node ID='x' type='BUTTON'><properties></properties><values></values><messages><midi><enabled>1</enabled></midi><osc><enabled>1</enabled></osc></messages></node></lexml>";
        let layout = parse_layout(xml).unwrap();
        assert_eq!(layout.root.messages.len(), 2);
        assert_eq!(layout.root.messages[0].raw, "<enabled>1</enabled>");
        let reserialized = serialize_layout(&layout);
        assert!(reserialized.contains("<midi><enabled>1</enabled></midi>"));
        assert!(reserialized.contains("<osc><enabled>1</enabled></osc>"));
    }

    #[test]
    fn omits_empty_children_and_messages_tags() {
        let xml = "<?xml version='1.0' encoding='UTF-8'?><lexml version='5'><node ID='x' type='GROUP'><properties></properties><values></values></node></lexml>";
        let layout = parse_layout(xml).unwrap();
        let out = serialize_layout(&layout);
        assert!(!out.contains("<children"));
        assert!(!out.contains("<messages"));
    }
}
