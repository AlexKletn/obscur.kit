Returns the caret's pen x (logical pixels) at byte `index` in `content`,
resolving the shaping inputs from the text node's computed `style` — the
**style-driven** wrapper over [`caret_offset_x`] the
runtime uses to keep the caret in view (single-line horizontal scroll, CSS
Overflow 3 §3.1). It resolves `font-family` / `font-weight` / `font-style`
/ `direction` / `font-size` **identically** to
[`caret_offset_at_x`] (the click inverse) and the
compose caret. `index == content.len()` yields the whole run advance (the
field's content width).

This single-line scroll measure sums the **un-spaced** shaped advances; the
compose caret + click inverse resolve their within-line x from the composed,
spaced/tabbed line ([`line_caret_x`]) instead. The two
agree exactly for the default field (no letter/word-spacing, no tab) that
horizontal scroll applies to; folding spacing into the scroll measure is
deferred to the horizontal-scroll item.
