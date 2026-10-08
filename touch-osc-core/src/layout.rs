use crate::error::{Error, Result};
use crate::tree::Layout;
use crate::{container, xml};
use std::fs;
use std::path::Path;

impl Layout {
    pub fn from_bytes(bytes: &[u8]) -> Result<Layout> {
        let xml_bytes = container::decompress(bytes)?;
        let xml_str = String::from_utf8(xml_bytes)
            .map_err(|e| Error::Malformed(format!("document is not valid UTF-8: {e}")))?;
        xml::parse_layout(&xml_str)
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Layout> {
        let bytes = fs::read(path)?;
        Layout::from_bytes(&bytes)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let xml_str = xml::serialize_layout(self);
        container::compress(xml_str.as_bytes())
    }

    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let bytes = self.to_bytes()?;
        fs::write(path, bytes)?;
        Ok(())
    }
}
