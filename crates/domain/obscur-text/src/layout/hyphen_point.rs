#[doc = include_str!("../../docs/api/layout/paragraph_word.doc/HyphenPoint.md")]
#[derive(Clone, Copy)]
pub(super) struct HyphenPoint {
    /// Exclusive glyph index the head runs up to.
    pub(super) head_end: usize,
    /// Glyph index the tail resumes at (`head_end`, or `head_end + 1` for a
    /// soft hyphen whose glyph is dropped).
    pub(super) tail_start: usize,
    /// The advance width of the head glyphs (`glyphs[0..head_end]`).
    pub(super) head_width: f32,
    /// The source byte offset the rendered hyphen glyph carries as its cluster.
    pub(super) break_cluster: usize,
}
