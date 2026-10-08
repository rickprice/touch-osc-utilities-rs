//! Node ID generation.
//!
//! `.tosc` node IDs are UUID strings. Fixtures show a mix of UUID
//! versions (v1-looking and v4-looking; see docs/FORMAT.md), so the
//! editor does not appear to require a particular version. This library
//! always generates v4 for new nodes.

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
