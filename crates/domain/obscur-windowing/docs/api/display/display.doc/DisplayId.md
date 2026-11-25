Stable, opaque identifier for a display.

`DisplayId` is a tightly-coupled part of [`Display`](super::Display): it identifies one
monitor and has no meaning apart from the display model, so it is colocated
here rather than given its own module. Backed by an integer handle; the
numeric value carries no meaning outside the subsystem that mints it.
