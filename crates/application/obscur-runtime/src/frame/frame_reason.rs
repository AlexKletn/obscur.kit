#![doc = include_str!("frame_reason.doc/_module.md")]

#[doc = include_str!("frame_reason.doc/FrameReason.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameReason {
    #[doc = include_str!("frame_reason.doc/Input.md")]
    Input,
    #[doc = include_str!("frame_reason.doc/TextEdit.md")]
    TextEdit,
    #[doc = include_str!("frame_reason.doc/UiCommit.md")]
    UiCommit,
    #[doc = include_str!("frame_reason.doc/Animation.md")]
    Animation,
    #[doc = include_str!("frame_reason.doc/ResourceCompletion.md")]
    ResourceCompletion,
    #[doc = include_str!("frame_reason.doc/WindowChange.md")]
    WindowChange,
    #[doc = include_str!("frame_reason.doc/ExternalRender.md")]
    ExternalRender,
    #[doc = include_str!("frame_reason.doc/DiagnosticsCapture.md")]
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
