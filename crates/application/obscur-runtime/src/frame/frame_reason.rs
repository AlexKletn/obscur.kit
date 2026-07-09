//! Why a frame is being produced: [`FrameReason`].

/// The reason a frame is being produced (spec §3.5).
///
/// Every frame is produced for exactly one reason, which lets the scheduler
/// coalesce and prioritise requests and lets diagnostics attribute work. The
/// set is closed and enumerated here; there is no string dispatch.
///
/// The type is a plain value: `Copy`, equatable, and hashable, so it can be used
/// as a map key or accumulated in a set of pending reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameReason {
    /// User input was received and must be dispatched (pointer, key, etc.).
    Input,
    /// A pure text edit into the focused editable that cannot change its box
    /// size (a same-size single-line keystroke, or any caret/selection move):
    /// only the field's glyphs + caret change, so the frame re-shapes and
    /// re-composes the field WITHOUT a restyle or relayout — the compose-only
    /// fast path (spec §3.4 phase 5). An edit that can resize the box (a
    /// wrapping/multiline field growing a line, an auto-width field) uses
    /// [`Input`](Self::Input) instead so its box relayouts.
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
