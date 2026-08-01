A clip restricting subsequent drawing to a region (spec §5.8).

A clip is either a geometric [`ClipShape`] or a [`Mask`]. Shape clips are
cheap: they confine drawing to a rectangle, rounded rectangle, or path.
Mask clips derive coverage from separate source content and may therefore
require an offscreen pass. A `Clip` is a backend-independent descriptor and
references no GPU or renderer types.
