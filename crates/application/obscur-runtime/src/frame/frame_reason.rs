//! Why a frame is being produced: [`FrameReason`].

#[doc = include_str!("../../docs/api/frame/frame_reason.doc/FrameReason.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameReason {
    /// User input was received and must be dispatched (pointer, key, etc.).
    Input,
    #[doc = include_str!("../../docs/api/frame/frame_reason.doc/TextEdit.md")]
    TextEdit,
    /// Application state committed, invalidating the tree and requiring a
    /// re-render.
    UiCommit,
    /// An animation or motion timeline advanced and needs a new sample.
    Animation,
    /// An asynchronous resource (image, font, data) completed loading.
    ResourceCompletion,
    /// The window or surface changed (resize, scale factor, visibility).
    WindowChange,
    /// An external driver requested a render (e.g. an embedding host).
    ExternalRender,
    /// A diagnostics or debugging capture requested a frame.
    DiagnosticsCapture,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn reasons_are_distinct_and_hashable() {
        let mut set = HashSet::new();
        for reason in [
            FrameReason::Input,
            FrameReason::TextEdit,
            FrameReason::UiCommit,
            FrameReason::Animation,
            FrameReason::ResourceCompletion,
            FrameReason::WindowChange,
            FrameReason::ExternalRender,
            FrameReason::DiagnosticsCapture,
        ] {
            assert!(set.insert(reason), "each reason must be distinct");
        }
        assert_eq!(set.len(), 8);
    }

    #[test]
    fn is_copy_and_eq() {
        let reason = FrameReason::Animation;
        let copy = reason;
        assert_eq!(reason, copy);
    }
}
