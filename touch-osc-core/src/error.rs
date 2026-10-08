use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("zlib decompression failed: {0}")]
    Decompress(io::Error),

    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("XML attribute error: {0}")]
    XmlAttr(#[from] quick_xml::events::attributes::AttrError),

    #[error("malformed .tosc document: {0}")]
    Malformed(String),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("validation failed:\n{}", .0.join("\n"))]
    Validation(Vec<String>),
}

pub type Result<T> = std::result::Result<T, Error>;
