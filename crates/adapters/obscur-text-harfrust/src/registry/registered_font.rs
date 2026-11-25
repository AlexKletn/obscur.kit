use std::sync::Arc;

use obscur_text::font::{FontAttributes, FontFamily, FontId, FontMetrics};
use read_fonts::{FontRef as ReadFontRef, TableProvider};

use super::is_ignorable;

/// One registered font: its id, requested family/attributes, raw bytes, and the
/// vertical metrics read once at registration time — `None` when the bytes carry
/// no readable `head`/`hhea`.
#[derive(Debug, Clone)]
pub(super) struct RegisteredFont {
    pub(super) id: FontId,
    pub(super) family: FontFamily,
    pub(super) attributes: FontAttributes,
    pub(super) bytes: Arc<[u8]>,
    pub(super) metrics: Option<FontMetrics>,
}

impl RegisteredFont {
    #[doc = include_str!("../../docs/api/registry.doc/covers_char.md")]
    pub(super) fn covers_char(&self, character: char) -> bool {
        let Ok(font) = ReadFontRef::new(&self.bytes) else {
            return false;
        };
        let Ok(cmap) = font.cmap() else {
            return false;
        };
        cmap.map_codepoint(character)
            .is_some_and(|gid| gid.to_u32() != 0)
    }

    pub(super) fn covers(&self, text: &str) -> bool {
        let Ok(font) = ReadFontRef::new(&self.bytes) else {
            return false;
        };
        let Ok(cmap) = font.cmap() else {
            return false;
        };
        text.chars()
            .filter(|ch| !is_ignorable(*ch))
            .all(|ch| cmap.map_codepoint(ch).is_some_and(|gid| gid.to_u32() != 0))
    }
}
