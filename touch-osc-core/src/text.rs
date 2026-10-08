//! Small helpers for decoding/encoding the raw XML text fragments stored
//! in [`crate::tree::Property`], [`crate::tree::ValueEntry`], etc.

/// Decode a raw `<![CDATA[...]]>`-wrapped or plain XML text fragment into
/// its string value.
pub fn decode(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(inner) = strip_cdata(trimmed) {
        inner.to_string()
    } else {
        quick_xml::escape::unescape(trimmed)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| trimmed.to_string())
    }
}

fn strip_cdata(s: &str) -> Option<&str> {
    let s = s.strip_prefix("<![CDATA[")?;
    s.strip_suffix("]]>")
}

/// Wrap a string value in a CDATA section, the form every `s`-type
/// property/key value uses in observed fixtures.
pub fn encode_cdata(s: &str) -> String {
    debug_assert!(
        !s.contains("]]>"),
        "CDATA content must not contain ']]>': {s:?}"
    );
    format!("<![CDATA[{s}]]>")
}

/// Format a float the way the editor does: whole numbers with no decimal
/// point (`1` not `1.0`), otherwise the shortest round-trip representation.
pub fn format_float(v: f64) -> String {
    if v.is_finite() && v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

/// Pull the float content out of a `<tag>...</tag>` sub-element inside a
/// raw XML fragment (used for `Color`/`Rect` components).
pub fn extract_tag_f64(raw: &str, tag: &str) -> Option<f64> {
    extract_tag_text(raw, tag)?.trim().parse().ok()
}

/// Pull the text content out of a `<tag>...</tag>` sub-element inside a
/// raw XML fragment.
pub fn extract_tag_text<'a>(raw: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = raw.find(open.as_str())? + open.len();
    let rest = &raw[start..];
    let end = rest.find(close.as_str())?;
    Some(&rest[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_cdata() {
        assert_eq!(decode("<![CDATA[background]]>"), "background");
    }

    #[test]
    fn decodes_plain_entities() {
        assert_eq!(decode("a &amp; b"), "a & b");
    }

    #[test]
    fn formats_whole_floats_without_decimal() {
        assert_eq!(format_float(1.0), "1");
        assert_eq!(format_float(0.0), "0");
    }

    #[test]
    fn formats_fractional_floats() {
        assert_eq!(format_float(0.25), "0.25");
    }

    #[test]
    fn extracts_tag_components() {
        let raw = "<r>0</r><g>0.25</g><b>1</b><a>1</a>";
        assert_eq!(extract_tag_f64(raw, "g"), Some(0.25));
    }
}
