An ordered chain of [`FontFamily`] values tried in turn when resolving text
(spec §5.18 fallback resolution, §5.10 font fallback).

The first family is the *primary* choice; later families are consulted only
when an earlier one cannot supply a needed glyph. The chain is kept
deduplicated: pushing a family already present is a no-op, so order is
preserved and the first occurrence wins.
