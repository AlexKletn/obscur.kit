#![doc = include_str!("../../docs/api/bootstrap/text_stack.doc/_module.md")]

use obscur_text::font::FontId;

#[doc = include_str!("../../docs/api/bootstrap/text_stack.doc/font_handle.md")]
pub(super) fn font_handle(id: FontId) -> u32 {
    u32::try_from(id.index()).unwrap_or(0)
}
