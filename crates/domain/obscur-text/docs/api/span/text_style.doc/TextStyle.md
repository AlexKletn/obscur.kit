The typography of a rich text span (spec §5.10).

A `TextStyle` describes how a run of text is drawn: its font family, size,
weight, slant, spacing, and line height. It is a pure, backend-agnostic value
type independent of the CSS/style domain; the spec's CSS typography
integration lives in an adapter that maps a computed style onto this type,
not the other way around.

Because it holds `f32` fields it derives only `Debug`, `Clone`, and
`PartialEq` (floats preclude `Eq`).
