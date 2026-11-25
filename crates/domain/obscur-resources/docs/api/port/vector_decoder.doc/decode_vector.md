Parses `bytes` into a vector document.

The result must keep `currentColor` unresolved
([`VectorPaint::CurrentColor`](obscur_scene::paint::VectorPaint::CurrentColor)),
so one parse serves every theme — §5.17's "theme/state changes without
reparsing" is a property of the decoder's output, not of its caller.

# Errors

Returns [`ResourceError::DecodeFailed`] when `bytes` are not a valid
document of this decoder's format.
