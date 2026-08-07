Everything a shaped paragraph depends on **except its text**: the font size,
the base direction, the requested fallback chain, the requested attributes,
and the identity of the default face the resolver falls back to.

Split out of the paragraph key so a cache probe costs no allocation. The
text is the part that varies from call to call — one entry per string — but
the style part barely varies at all: a screen typically has a handful of
distinct (size, chain, attributes) combinations and nothing changes them
between frames. Storing it once per *group* of paragraphs rather than once
per paragraph means a lookup can compare it against the caller's borrowed
[`FontFallback`] / [`FontFace`] in place — see [`matches`](Self::matches) —
instead of cloning every family name into an owned key just to hash it. The
old whole-key form allocated a `Box<str>` for the text plus one for every
family name and one for the default face's source **on every probe**,
including the hits, which on a scrolling list is once per visible text run
per frame.

Deliberately not `Hash`: it is found by linear scan over the few live
groups, which is what lets the comparison take borrowed inputs.
