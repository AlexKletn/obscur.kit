The orchestrator that shapes a whole paragraph across multiple fonts
(spec §5.10 font fallback).

A single [`TextShaper`] only shapes one uniformly-styled, single-font run;
real paragraphs mix scripts and therefore fonts. `ParagraphShaper` composes
three ports to bridge that gap: it [`itemize`](TextItemizer::itemize)s the
text into uniform runs, [`resolve`](FontResolver::resolve)s a covering font
for each, and [`shape`](TextShaper::shape)s each run with the resolved face,
yielding one [`ShapedRun`] per item in logical order.

Each sub-run is shaped against a *slice* of the paragraph, so its glyph
clusters and source range come back relative to that slice (0-based).
`ParagraphShaper` rewrites them into **full-text byte coordinates** — every
glyph's cluster is offset by the item's start and the run's source is set to
the item's range — so callers can map any glyph straight back to the original
string.

# Paragraph shaping cache

The runtime shapes each text node twice per frame — once to **measure**
(layout) and once to **compose** (scene) — and re-shapes every frame even
when nothing changed, and the *whole* itemize + font-resolve + multi-run
assembly re-runs each time. `ParagraphShaper` therefore memoizes each
[`shape_paragraph`](Self::shape_paragraph) result behind an interior
[`RefCell`] cache keyed by every input that affects the output — the
paragraph's text, and the style it is shaped under. Because the result is width-independent, measure and
compose share the same cached runs, so an unchanged node itemizes, resolves,
and shapes exactly **once** per frame and hits the cache on later frames. The
output is byte-identical to shaping without the cache — a pure performance
change. Swapping any port ([`set_itemizer`](Self::set_itemizer),
[`set_resolver`](Self::set_resolver), [`set_shaper`](Self::set_shaper))
clears the cache, since the memoized runs no longer reflect the ports.

The cache is single-threaded interior state (`RefCell`): the runtime shapes
on one thread and the ports carry no `Send`/`Sync` bound.
