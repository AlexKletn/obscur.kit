A half-open byte range `[start, end)` into a UTF-8 text buffer (spec §5.10).

Rich text is addressed by byte offsets, not char or grapheme indices: a
`TextRange` names the bytes `start..end` of some text buffer. Callers are
responsible for keeping `start` and `end` on UTF-8 character boundaries; this
type performs no boundary validation and treats offsets as opaque `usize`
positions.

The range is always normalized so that `start <= end`. Constructors that
receive an inverted range clamp `end` up to `start` (see [`TextRange::new`]),
so `len` never underflows.
