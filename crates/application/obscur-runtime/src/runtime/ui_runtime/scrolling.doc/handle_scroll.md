Routes a wheel/scroll gesture of `delta_y` logical px at host `position` to
the **innermost** scroll container under the pointer, scrolling it (clamped)
and requesting an [`Input`](FrameReason::Input) frame. Returns the scrolled
container's [`NodeId`], or [`None`] when no scroll container lies under the
pointer (CSS Overflow 3 §scroll-container).

`position` is in the host's physical (device) space; it is mapped to content
space through the same `physical / device_scale + content_offset` inverse
[`handle_pointer`](Self::handle_pointer) uses, then through each crossed
container's own `scroll_top`, so the hit-tested container is the one the
pointer visually sits over while scrolled.
