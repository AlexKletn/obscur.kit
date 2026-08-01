Narrows the computed `z-index` number (an already-integral `<integer>`, stored
as `f32`) into the `i32` stacking level.

The CSS lowering only admits integral numbers, so truncation is exact for the
modelled range; a pathological out-of-range value saturates rather than
wrapping.
