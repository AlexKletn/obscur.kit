//! The immutable inputs for one frame: [`FrameContext`].

use obscur_geometry::Size;
use obscur_style::property::EnvironmentValues;

use super::{FrameId, FrameReason};

#[doc = include_str!("../../docs/api/frame/frame_context.doc/FrameContext.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameContext {
    id: FrameId,
    reason: FrameReason,
    viewport: Size,
    environment: EnvironmentValues,
}

impl FrameContext {
    /// Creates a context for the frame `id`, produced for `reason`, laid out
    /// under `viewport`.
    #[must_use]
    pub const fn new(id: FrameId, reason: FrameReason, viewport: Size) -> Self {
        Self {
            id,
            reason,
            viewport,
            environment: EnvironmentValues::none(),
        }
    }

    #[doc = include_str!("../../docs/api/frame/frame_context.doc/with_environment.md")]
    #[must_use]
    pub const fn with_environment(mut self, environment: EnvironmentValues) -> Self {
        self.environment = environment;
        self
    }

    /// The host-reported environment values in force for this frame (spec §6.15).
    #[must_use]
    pub const fn environment(self) -> EnvironmentValues {
        self.environment
    }

    /// Returns the identity of this frame.
    #[must_use]
    pub const fn id(self) -> FrameId {
        self.id
    }

    /// Returns the reason this frame is being produced.
    #[must_use]
    pub const fn reason(self) -> FrameReason {
        self.reason
    }

    #[doc = include_str!("../../docs/api/frame/frame_context.doc/viewport.md")]
    #[must_use]
    pub const fn viewport(self) -> Size {
        self.viewport
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_return_constructor_inputs() {
        let ctx = FrameContext::new(
            FrameId::from_raw(3),
            FrameReason::UiCommit,
            Size::new(800.0, 600.0),
        );
        assert_eq!(ctx.id(), FrameId::from_raw(3));
        assert_eq!(ctx.reason(), FrameReason::UiCommit);
        assert_eq!(ctx.viewport(), Size::new(800.0, 600.0));
    }

    #[test]
    fn is_copy_and_eq() {
        let ctx = FrameContext::new(FrameId::FIRST, FrameReason::Input, Size::ZERO);
        let copy = ctx;
        assert_eq!(ctx, copy);
    }
}
