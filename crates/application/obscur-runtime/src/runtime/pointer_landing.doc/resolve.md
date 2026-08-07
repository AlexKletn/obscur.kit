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
container's clamped `scroll_top` is collected here and handed to the hit
test, which adds back — per candidate — only the ones that candidate
actually rides. Folding the point ONCE, globally, and matching the result
against every node is what this replaced: a fixed toolbar or an absolutely
positioned overlay that merely OVERLAPS a scrollport was tested at
coordinates shifted by a scroll it does not participate in, and could not
be clicked. With no scroll container the map is empty and the point is
unchanged, so the hit test is byte-identical for existing hosts.

The **screen** point is retained as well: it is the space a fixed box and
every scrollport clip are stated in, and the space the caret seat and drag
re-select measure against.

# 3. Capture, else hit test

The topmost node whose box contains the point, respecting overflow
clipping, `border-radius` shape, and stacking order (§4.7) — unless this
pointer is captured, in which case its captor is the target wherever the
pointer actually is (§2.4: capture overrides hit-testing). That is the
whole purpose of capture: a control that has claimed a gesture keeps it
even when the pointer wanders off, which is the difference between a drag
that works past the edge of its thumb and one that dies there.
