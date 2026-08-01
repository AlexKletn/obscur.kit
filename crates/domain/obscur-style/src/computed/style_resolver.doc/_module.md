Turns cascaded values into computed values: [`StyleResolver`] (§4.13–§4.14).

The resolver implements the computed-value algorithm for a single node:

1. **Effective custom properties** — custom properties always inherit, so the
   node's effective set is the parent's set overridden by the node's own
   cascaded custom properties. This set is resolved *before* any computed
   value, and is stored on the resulting [`ComputedStyle`] so descendants
   inherit it (§4.14).
2. **`var()` substitution** — a [`PropertyValue::CustomRef`] is replaced by the
   referenced effective custom property, following chains with a visited-set
   guard so a cycle terminates. An unresolved reference (missing name or a
   cycle) resolves as if the declaration were [`PropertyValue::Unset`].
3. **`font-size`, before anything else** — it is the base its own node's `em`
   resolves against, so it cannot be folded with the context it defines. It is
   resolved against the *parent's* size (which is what `em` and `%` mean on
   this property) and folded to absolute pixels first; see
   [`absolutize::absolute_font_size`].
4. **Per-property computed value** — the (substituted) cascaded value is
   resolved against the CSS-wide keywords and default inheritance to produce a
   concrete [`ComputedValue`], and then **absolutized**: every `<length>` it
   carries is folded to `px` against the node's own [`LengthContext`] (§4.14 —
   the computed value of a `<length>` is absolute). Downstream consumers read
   pixels; none of them needs a length context.

Each piece this pass orchestrates lives beside it: the property set it walks
(`computed_property_set`), the initial values it falls back to
(`initial_value`), steps 1–2 themselves (`effective_custom_properties`), the
§4.13 choice between a logical declaration and its physical counterpart
(`logical_cascade`), the relative `font-weight` table (`relative_font_weight`)
and step 4's length fold (`absolutize`). What is left here is the pass.

# Deliberate limitations

* **Keyword initial values are deferred.** The initial values of keyword-
  valued properties (`display: inline`, `visibility: visible`) are interned
  symbols, and this crate has no interner in Phase 1. Rather than fake one,
  [`StyleResolver::initial_value`] returns [`None`] for those properties, and a
  property that would need its (unavailable) initial value is simply left
  unresolved on the [`ComputedStyle`]. A later phase supplies the interner and
  fills these in.
* **Full modelled property set.** The resolver computes *every* modelled
  [`PropertyId`] longhand (see [`StyleResolver::computed_properties`]), so an
  authored value for any of them flows through the computed-value step to the
  downstream layout / scene / text consumers. A property whose only value would
  be a deferred keyword initial is still left unresolved (see the previous
  bullet); an *unauthored* node is therefore byte-for-byte unchanged, because
  each property still resolves to exactly the value (or [`None`]) it did before
  — the set merely no longer drops authored values for the added longhands.
* **`ch` absolutizes only when the caller supplies metrics.** Every other unit
  folds from numbers this step already holds; `ch` is the advance of the `0`
  glyph, which needs text *shaped* in the node's own face, and this crate has
  no shaper and should not have one. It therefore *asks*, through the
  [`ChMetrics`](super::ChMetrics) port on [`ResolutionContext`] — the
  measurement is threaded into computed-value time rather than deferred to
  whichever consumer happens to own a shaper. A caller that attaches one gets
  `ch` folded to `px` like every other unit, and paint reads it correctly.

  A caller that attaches **no** port leaves `ch` symbolic on the
  [`ComputedStyle`], exactly as before this port existed: this step will not
  bake in [`LengthContext::ch_base`]'s `0.5em` substitute, which is wrong for
  exactly the monospace faces `ch` exists to serve. Layout can still measure
  such a term; paint cannot, so on that path `ch` still reaches paint as `0`.
  The seam is closed for callers that supply metrics, not by construction.
