Maps `event`'s position into content space, folds it through the scroll
containers it lies within, and resolves the node it addresses.

# 1. Device scale + content scroll

Incoming coordinates are in the host's physical (device) space, but layout
runs in logical CSS px scrolled by `content_offset`. The pointer is mapped
to content space once, here, as `physical / device_scale + offset` — the
inverse of the root render transform `scale(s) · translate(-ox, -oy)` — so
every downstream consumer (the hit test below, the caret seat, the drag
re-select) sees content-space coordinates. At scale `1.0` and offset
`(0, 0)` this is the identity, so the mapping is byte-identical for
existing hosts.

# 2. Per-element scroll

The spatial index is keyed by each node's UNSCROLLED layout box, but a
scroll container's descendants render translated by `−scroll_top`. Each
crossed container's `scroll_top` is added back so a click while scrolled
hit-tests the scrolled content (CSS Overflow 3 §scroll-container). With no
scroll container this is the identity, so the hit test is byte-identical
for existing hosts.

The **screen** point (before this per-element scroll offset is added) is
retained: a scroll container clips its content to the scrollport ON
SCREEN, so the clip test in [`HitTesting`] is against the screen point
while box containment / shape is against the scroll-mapped point (the
space the unscrolled boxes live in).

# 3. Capture, else hit test

The topmost node whose box contains the point, respecting overflow
clipping, `border-radius` shape, and stacking order (§4.7) — unless this
pointer is captured, in which case its captor is the target wherever the
pointer actually is (§2.4: capture overrides hit-testing). That is the
whole purpose of capture: a control that has claimed a gesture keeps it
even when the pointer wanders off, which is the difference between a drag
that works past the edge of its thumb and one that dies there.
