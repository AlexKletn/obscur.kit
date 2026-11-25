#[doc = include_str!("../../docs/api/clipboard/clipboard_request.doc/ClipboardWriteMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardWriteMode {
    /// Copy: the source keeps its content (spec §6.20 COPY).
    Copy,
    /// Cut: the source should delete its content after a successful read
    /// (spec §6.20 CUT).
    Cut,
}

impl ClipboardWriteMode {
    /// Returns whether this is [`ClipboardWriteMode::Cut`] (spec §6.20 CUT).
    #[must_use]
    pub const fn is_cut(self) -> bool {
        matches!(self, Self::Cut)
    }
}

#[cfg(test)]
mod tests {
    use super::ClipboardWriteMode;

    #[test]
    fn write_mode_predicates_classify_cut() {
        assert!(ClipboardWriteMode::Cut.is_cut());
        assert!(!ClipboardWriteMode::Copy.is_cut());
    }
}
