The surface frames are laid out and painted into: its extent, its density, and
the two offsets that place the document inside it.

# Why these four are one entity

They are one coordinate system, and the code that gets it wrong gets it wrong
in *pairs*:

- [`root_transform`](Self::root_transform) and
  [`to_content_point`](Self::to_content_point) must stay exact inverses. The
  first wraps the composed scene so it rasterizes at physical resolution and
  scrolled to the current offset; the second maps a host pointer position back
  into the space layout happened in. Written apart, a change to one silently
  moves hit-testing off the pixels it is testing against — a defect nothing
  type-checks and no single test names.
- [`viewport`](Self::viewport) and [`device_scale`](Self::device_scale) are
  derived from one another by [`set_surface`](Self::set_surface): the viewport
  is the physical extent divided by the *clamped* scale, so a host cannot end
  up with a viewport computed from a scale the runtime rejected.

# What it deliberately does not do

It has no view of the tree, the scene or the caches. A scale change that
selects different asset files is *reported* by
[`set_device_scale`](Self::set_device_scale) rather than acted on, because
dropping the decoded-image cache belongs to whoever owns that cache.
