CSS Text 3 `letter-spacing` (§8.1) and `word-spacing` (§8.2): post-shaping
advance adjustments applied to positioned runs.

Spacing is deliberately modeled as a **pure, width-independent** rewrite of a
shaped paragraph's glyph advances, sitting *between* the (cached)
[`shape_paragraph`](crate::shaping::ParagraphShaper::shape_paragraph) result
and the width-dependent line wrap. Because it is one function, the runtime's
measure pass and its compose pass call the *same* seam with the *same* values,
so wrapping (which measures the spaced advances) and painting can never drift.

Two spec rules, both additive and both deterministic:

* **letter-spacing (§8.1)** — "additional spacing (commonly called *tracking*)
  between adjacent typographic character units". The resolved length is added
  *after* every typographic character unit in the run. A typographic character
  unit is a grapheme cluster; lacking a grapheme segmenter here we approximate
  it by the shaper's **cluster** grouping (one add per distinct source
  cluster), which is exact for the common one-char-per-cluster case and is a
  documented refinement for combining sequences. The trailing letter-spacing
  at the end of a line is meant to *hang* (be trimmed); this pass adds it
  uniformly (after every cluster incl. the last), and the per-line trim — which
  §8.1 says is "deferred to the line breaker" — is performed by the wrapper
  (`ParagraphLayout::wrap_core` → `PackedLine::hang_trailing`), the only stage that
  knows where lines end.

* **word-spacing (§8.2)** — "additional spacing between 'words'". The resolved
  length is added to the advance of each **word separator**. Per the spec the
  word-separator set is small; we honor the two that occur in normal text:
  U+0020 SPACE and U+00A0 NO-BREAK SPACE. (The Ethiopic/Aegean separators the
  spec also lists are a documented omission.)

Both lengths may be **negative** (§8.1/§8.2 allow it); the result is *not*
clamped here, so a tight negative tracking can produce a small or negative
per-glyph advance — the scene and the CPU/Vello backends accumulate advances
as a signed pen, so this renders correctly. When both lengths are zero the
input runs are returned untouched, byte-for-byte.
