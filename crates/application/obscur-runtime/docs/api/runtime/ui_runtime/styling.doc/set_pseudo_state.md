Temporarily replaces the node’s interaction pseudo-state. Runtime updates to
hover, focus or press rebuild the projection and replace manual states. Use
the `CHECKED` and `DISABLED` view attributes for application-owned state.

Call [`request_frame`](Self::request_frame) with [`FrameReason::Input`] to
schedule the change for paint.

[`FrameReason::Input`]: crate::frame::FrameReason::Input
