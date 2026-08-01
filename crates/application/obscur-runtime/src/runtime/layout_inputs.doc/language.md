The locale's language subtag, which every text leaf was shaped — and
therefore measured — under (spec §7.2).

A layout input and not a computed style: a shaper may substitute
language-sensitively (`locl`), and a substituted glyph may be a different
width, so the same string under `ru` and `sr` can measure to different
sizes. Nothing in the per-node style diff can see that, which is exactly
why it belongs here — a locale switch must fall through to a full layout
rather than reuse boxes measured under the outgoing language.
