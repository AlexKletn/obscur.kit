Sets the content scroll `offset` `(x, y)` in **logical** CSS pixels — the
content point pinned to the viewport top-left. A positive offset scrolls
content up/left (spec §4.1).

The offset is composed with [`device_scale`](Self::device_scale) into the
scene's single root transform `scale(s) · translate(-ox, -oy)`, and added
back to pointer coordinates in [`handle_pointer`](Self::handle_pointer)
(`physical / s + offset`) so hit-testing stays correct while scrolled.
Non-finite components are ignored (the previous value is kept for that
axis). Setting `(0, 0)` at scale `1.0` restores the byte-identical
(untransformed) path.

Does not itself schedule a frame; a host typically pairs this with
[`request_frame`](Self::request_frame)`(`[`FrameReason::WindowChange`]`)`.

[`FrameReason::WindowChange`]: crate::frame::FrameReason::WindowChange
