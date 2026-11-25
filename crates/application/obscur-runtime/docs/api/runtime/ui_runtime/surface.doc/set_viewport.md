Sets the viewport subsequent frames are laid out under.

Does not itself schedule a frame; a host typically pairs this with
[`request_frame`](Self::request_frame)`(`[`FrameReason::WindowChange`]`)`.

[`FrameReason::WindowChange`]: crate::frame::FrameReason::WindowChange
