Finds the line-break opportunities in `text` under a `white_space` mode
(CSS Text 3 §3 white-space table, §5.10).

This is a deliberately **simplified** approximation of the Unicode line
breaking algorithm (UAX #14); full, tailored breaking is an adapter
concern. The `white_space` mode selects which breaks a run yields:

- a [`Hard`](BreakKind::Hard) break just after each `'\n'` (its offset is
  the byte index of the character following the newline) — but **only**
  when [`white_space.preserves_segment_breaks()`](WhiteSpace::preserves_segment_breaks)
  (`pre` / `pre-wrap` / `pre-line`). Under `normal` / `nowrap` a segment
  break has already been collapsed to a space at the pre-shape seam, so it
  is treated here as ordinary whitespace;
- a [`Soft`](BreakKind::Soft) break after each maximal run of whitespace,
  at the byte offset of the first non-whitespace character that follows it
  (the break sits *before* the next word) — but **only** when
  [`white_space.wraps()`](WhiteSpace::wraps) (`normal` / `pre-wrap` /
  `pre-line`). Under `nowrap` / `pre` no soft break is emitted, so the
  wrapper lays each segment on a single (possibly overflowing) line.

A whitespace run that contains a preserved segment break yields only the
mandatory break, never a duplicate soft break at the same position.
Offsets are byte-correct for UTF-8 because iteration is by
[`str::char_indices`]. The returned breaks are in ascending offset order.
