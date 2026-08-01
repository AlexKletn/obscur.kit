Greedily wraps a sequence of font-tagged shaped `runs` of `text` into a
multi-font paragraph no wider than `max_width` (spec §5.10).

The `runs` are the logical-order output of the font-fallback orchestrator;
their glyph clusters are byte offsets into the full `text`. The algorithm:

1. flattens `runs` into one ordered glyph stream, tagging each glyph with
   its run's font, direction, and font size;
2. splits the stream into words at the [`LineBreak::opportunities`] of
   `text`, mapping each break's byte offset to the glyph whose cluster
   equals it — a word may therefore span several fonts;
3. greedily packs words onto lines under `max_width`, closing a line when
   the next word would overflow a non-empty line and always after a
   mandatory break; a word wider than `max_width` still takes its own line;
4. splits each line's glyphs into contiguous same-font, same-level
   segments, rebuilding a [`ShapedRun`] per segment, reorders those
   segments into visual order via the UAX#9 L2 rule (see
   [`line_segments`]), and assigns the line its cumulative `top` (spaced by
   `line_height`), `height`, `baseline`, and alignment offset.

A **non-finite** `max_width` (`f32::INFINITY`) is the explicit *no
constraint* sentinel (CSS 2.1 §10.8 available-width; the intrinsic /
max-content measure): the whole paragraph is one line. A **finite**
`max_width >= 0` is a real box, so a `0`/tiny width max-wraps toward
min-content rather than overflowing on one line. The
`white_space` mode (CSS Text 3 §3) selects the break opportunities: a
non-wrapping mode (`nowrap` / `pre`) yields no soft breaks, so each segment
between mandatory (`\n`) breaks lays out on a single line that may overflow
`max_width` — which is correct (overflow/ellipsis is a separate concern).
Empty input — no runs, or a single empty run — yields one empty line rather
than panicking. This greedy wrapper reorders a line's *segments* into
visual order (line-level bidi) but never reorders glyphs within a segment —
that is the shaper's job — and never splits a word, even one that spans
fonts.
