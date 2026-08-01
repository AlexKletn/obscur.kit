The typed outcome of presenting a [`FrameSubmission`](crate::frame::FrameSubmission)
(spec §5.2).

Presentation outcomes are explicit diagnostics rather than silent failures
(spec §5.2): a frame either reaches the screen, is dropped as a stale
intermediate frame (spec §3.5), triggers device loss (see
[`DeviceLoss`](crate::frame::DeviceLoss)), or fails outright with a reason.
