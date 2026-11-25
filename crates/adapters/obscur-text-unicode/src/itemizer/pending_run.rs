use obscur_text::shaping::{ScriptTag, TextDirection, TextItem};
use obscur_text::span::TextRange;

/// The run currently being accumulated during a single left-to-right pass.
pub(super) struct PendingRun {
    /// Byte offset where the run starts.
    pub(super) start: usize,
    /// The run's uniform bidi embedding level.
    pub(super) level: u8,
    /// The run's resolved script, or `None` while only neutral characters have
    /// been seen (a leading neutral prefix awaiting a concrete script).
    pub(super) script: Option<ScriptTag>,
}

impl PendingRun {
    /// Finalizes the run as a [`TextItem`] spanning `self.start..end`.
    pub(super) fn finish(&self, end: usize) -> TextItem {
        let direction = if self.level.is_multiple_of(2) {
            TextDirection::LeftToRight
        } else {
            TextDirection::RightToLeft
        };
        TextItem::new(
            TextRange::new(self.start, end),
            self.script.unwrap_or(ScriptTag::COMMON),
            direction,
            self.level,
        )
    }
}
