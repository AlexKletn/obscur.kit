The one pass that makes a computed `<length>` absolute (§4.14).

CSS specifies that the *computed* value of a `<length>` is already absolute:
`em` / `rem` / `vw` / `vh` are folded once, at the computed-value step, and
every downstream consumer reads plain pixels. This module is that fold —
[`absolutize_value`] walks a [`PropertyValue`] and rewrites every [`Length`]
it carries through [`Length::absolutize`], however deeply nested.

Doing it here, once, is the whole point: before this pass existed layout
folded relative units itself (via `Length::resolve_px`) while paint read the
same property through `Length::as_px`, which honours absolute `px` only. The
two phases therefore disagreed about the same box — `border-width: .1em`
reserved space and painted nothing.

# What stays symbolic, and why

* **Percentages.** A `%` resolves against a containing block that layout has
  not produced at computed-value time, so it survives the pass untouched
  (the one exception is `font-size: 150%`, whose basis *is* known — the
  parent's font-size — and which [`StyleResolver`](super::StyleResolver)
  folds itself before building this pass's context).
* **`auto`.** It has no length at all until layout picks the used value.
* **`ch` with no measurement.** See [`Length::absolutize`].
