One flattened glyph tagged with the font, direction, font size, and bidi
embedding level of the [`ShapedRun`] it came from. Flattening the fallback
result into this stream lets word-splitting and greedy packing treat the
whole paragraph uniformly while retaining enough per-glyph provenance to
rebuild same-font, same-level segments and reorder them into visual order.
