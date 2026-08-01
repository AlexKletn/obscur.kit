A half-open byte range `[start, end)` within a CSS source (spec §2.5).

Spans locate tokens, values, and diagnostics inside a [`crate::source::CssSource`]
by byte offset. The invariant `end >= start` always holds: [`SourceSpan::new`]
clamps an inverted `end` up to `start`, yielding an empty span rather than a
negative range. Offsets are byte positions, not character positions, matching
how the parser walks the source text.
