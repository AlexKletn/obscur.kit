use std::collections::HashMap;
use std::sync::Arc;

use crate::shaping::{ShapedRun, paragraph_style_key::ParagraphStyleKey};

#[doc = include_str!("../../docs/api/shaping/shape_cache.doc/StyleGroup.md")]
pub(super) struct StyleGroup {
    /// The style every paragraph in this group was shaped under.
    pub(super) style: ParagraphStyleKey,
    /// Paragraph text to its shaped runs. `Box<str>` keys so a `&str` probes the
    /// map without allocating a key first.
    pub(super) texts: HashMap<Box<str>, Arc<[ShapedRun]>>,
}
