The three byte-offset tables the greedy wrap asks about the **source text**,
collected once per wrap and consulted per glyph cluster (spec §5.10).

Each answers one question about a cluster, and all three are derived from the
same `text` and the same finalized break list, which is why they are collected
together rather than threaded as three loose sets:

- [`is_hyphen_point`](Self::is_hyphen_point) — may a word be split here, with
  a rendered hyphen (the hyphenator's proposal);
- [`is_soft_hyphen`](Self::is_soft_hyphen) — did the author write a `U+00AD`
  here, so a split may drop that glyph and render a hyphen in its place;
- [`hangs`](Self::hangs) — is this a collapsible space that hangs when it ends
  a line (CSS Text 3 §3/§7.1).
