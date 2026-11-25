An owned CSS source together with its identity and text (spec §2.5).

A `CssSource` bundles a [`SourceId`], a logical `name` (for example
`"theme.css"`), and the full source `text`. The parser reads `text` and
produces [`SourceSpan`]s into it; [`CssSource::slice`] resolves a span back
to the substring it covers. Both `name` and `text` are stored as `Box<str>`
so a source is compact and immutable once built.
