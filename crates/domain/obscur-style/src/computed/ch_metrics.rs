//! The one input the computed-value step cannot compute for itself:
//! [`ChMetrics`].
//!
//! Every relative unit CSS defines folds from numbers the resolver already holds
//! — `em` from the node's `font-size`, `rem` from the root's, `vw`/`vh` from the
//! viewport. `ch` is the exception, and the only one: CSS Values 4 §6.1.2 defines
//! it as *"the used advance measure of the `0` (ZERO, U+0030) glyph in the font
//! used to render it"*. That is a measurement, not an arithmetic — it needs a
//! font loaded and a glyph shaped, which is a capability no domain crate has or
//! should have.
//!
//! So the resolver asks for it through this port rather than computing it. The
//! measurement is *threaded into* computed-value time instead of being deferred
//! to a consumer that happens to own a shaper — which is what made `ch` the last
//! unit to die at the layout↔paint seam: layout could measure and did, paint
//! could not and read `0`.
//!
//! # Why the port takes a whole [`ComputedStyle`]
//!
//! The `0` advance depends on the font actually selected — `font-family`, and the
//! `font-weight`/`font-style` that pick the face within it — as well as the size.
//! Rather than invent a font-description struct this crate would have to keep in
//! sync with the real one, the port takes the style that names them. The resolver
//! guarantees the style it passes already carries the properties an implementor
//! needs to select a face; see [`ChMetrics::zero_advance`].

use crate::computed::ComputedStyle;

/// Measures the font metrics the computed-value step needs but cannot derive.
///
/// Implemented by whatever layer owns a text shaper (in this workspace, the
/// runtime's paragraph orchestrator) and handed to the resolver through
/// [`ResolutionContext::with_metrics`](super::ResolutionContext::with_metrics).
///
/// An implementor is a *measurement*, not a policy: it reports what the font says
/// and returns [`None`] when it cannot, leaving the specified fallback to the
/// caller. It must not substitute `0.5em` itself — see [`Self::zero_advance`].
pub trait ChMetrics {
    /// The advance of the `0` (ZERO, U+0030) glyph for the font `font` selects,
    /// in logical pixels — the base every `ch` length resolves against.
    ///
    /// `font` is guaranteed to carry the node's already-resolved `font-size`
    /// (absolute px) plus its `font-family` / `font-weight` / `font-style`, which
    /// is everything needed to select a face. It carries nothing else that can be
    /// relied on: the resolver calls this *mid-resolution*, before the remaining
    /// properties are computed, precisely because their `ch` terms depend on the
    /// answer. An implementor that reads any other property is reading a value
    /// that is not there yet.
    ///
    /// Returns [`None`] when the advance cannot be measured — no face, or a face
    /// with no `0`. [`None`] is the honest answer and is **not** the same as
    /// `Some(0.0)`: it routes the caller to
    /// [`LengthContext::ch_base`](crate::property::LengthContext::ch_base)'s
    /// specified `0.5em` substitute (§6.1.2: *"in the cases where it is impossible
    /// or impractical to determine the measure of the `0` glyph, it must be
    /// assumed to be 0.5em wide"*), whereas `Some(0.0)` would silently collapse
    /// `width: 60ch` to nothing.
    ///
    /// That distinction holds **at layout only**. Paint leaves an unmeasured `Ch`
    /// symbolic and reads it as 0, so there `None` and `Some(0.0)` are
    /// indistinguishable — which is the seam this port narrows rather than closes:
    /// a family that resolves to no face still lays out against the 0.5em
    /// substitute and paints at 0.
    fn zero_advance(&self, font: &ComputedStyle) -> Option<f32>;
}
