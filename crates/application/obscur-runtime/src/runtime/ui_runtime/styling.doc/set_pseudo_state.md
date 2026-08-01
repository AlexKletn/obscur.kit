Records `state` as `node`'s interaction state for subsequent frames.

This does **not** itself schedule a frame: the caller (the input step) is
responsible for calling [`request_frame`](Self::request_frame) — typically
with [`FrameReason::Input`] — so the state change actually reaches paint.
Keeping the two explicit avoids hidden frame requests.

[`FrameReason::Input`]: crate::frame::FrameReason::Input
