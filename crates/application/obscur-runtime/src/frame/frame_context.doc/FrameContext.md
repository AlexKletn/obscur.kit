The immutable inputs that parameterise one frame's pipeline run.

A `FrameContext` bundles the identity ([`FrameId`]), the reason the frame is
being produced ([`FrameReason`], spec §3.5), and the viewport the frame is
laid out under. It is read-only for the duration of the frame: the pipeline
consumes it but never mutates it, which keeps the synchronous frame phase
deterministic (spec §3.4).

The type is a small plain value (`Copy`), so it can be threaded freely
through the pipeline without borrowing concerns.
