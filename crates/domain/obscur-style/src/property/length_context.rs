//! The resolution context a relative length is evaluated against.
//!
//! Font-relative units (`em` / `rem` / `ch`), viewport units (`vw` / `vh` /
//! `vmin` / `vmax`) and `calc()` / `min()` / `max()` / `clamp()` expressions carry no
//! absolute value on their own: they only become a concrete number once a
//! [`LengthContext`] supplies the bases they resolve against (spec §4.5).
//!
//! This context serves both of the folds that need those bases:
//!
//! * the **computed-value** fold
//!   ([`Length::absolutize`](crate::property::Length::absolutize)), which runs
//!   once per node and retires `em` / `rem` / `vw` / `vh` for good — after it,
//!   no consumer of a computed style needs a context to read a length; and
//! * the **use-time** fold
//!   ([`Length::resolve_px`](crate::property::Length::resolve_px) /
//!   [`LengthExpr::resolve`](crate::property::LengthExpr::resolve)), which layout
//!   still runs for the terms the first fold cannot absolutize — a `%` (it needs
//!   [`percent_basis`](LengthContext::percent_basis), which only layout knows)
//!   and a `ch` (it needs [`ch_width`](LengthContext::ch_width), which only a
//!   caller with a shaper can measure).

use super::EnvironmentValues;

/// The bases a relative length resolves against.
///
/// * `font_size` — the element's own computed `font-size`, in logical pixels;
///   the base for `em`.
/// * `root_font_size` — the document root's computed `font-size`, in logical
///   pixels; the base for `rem`.
/// * `viewport_width` / `viewport_height` — the viewport dimensions in logical
///   pixels; the bases for `vw` / `vh` (and, combined, `vmin` / `vmax`).
/// * `ch_width` — the advance of the `0` (ZERO, U+0030) glyph in the element's
///   own font, in logical pixels; the base for `ch`. It is [`None`] when that
///   advance could not be measured, in which case [`ch_base`](Self::ch_base)
///   substitutes the specified fallback — see there.
/// * `percent_basis` — the length a `%` resolves against, when it is known at
///   resolution time. It is [`None`] when the basis is only known later (bare
///   percentages are resolved by layout against the containing block, not here),
///   in which case a `%` term cannot be folded and resolution returns [`None`].
///
/// Because every field is an `f32` (or an `Option<f32>`), this type derives
/// [`PartialEq`] but not [`Eq`]/[`Hash`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LengthContext {
    /// The element's own computed `font-size` in logical pixels (base for `em`).
    pub font_size: f32,
    /// The document root's computed `font-size` in logical pixels (base for
    /// `rem`).
    pub root_font_size: f32,
    /// The viewport width in logical pixels (base for `vw`).
    pub viewport_width: f32,
    /// The viewport height in logical pixels (base for `vh`).
    pub viewport_height: f32,
    /// The measured advance of the `0` glyph (base for `ch`), if it is known.
    /// [`None`] means "not measurable here" — see [`ch_base`](Self::ch_base).
    pub ch_width: Option<f32>,
    /// The basis a `%` resolves against, if known at resolution time.
    pub percent_basis: Option<f32>,
    /// The host-reported environment values `env()` reads (spec §6.15). All
    /// zeroes unless a host set them — which is the right answer for a screen
    /// with no cutout and no on-screen keyboard, not a placeholder for one.
    pub environment: EnvironmentValues,
}

impl LengthContext {
    /// Constructs a context with an unknown percentage basis and an unmeasured
    /// `ch`.
    ///
    /// `em` / `rem` and the viewport-relative units resolve; a `%` term does not
    /// (it is resolved later, by layout, against the containing block), and `ch`
    /// falls back to [`ch_base`](Self::ch_base)'s substitute until a caller that
    /// can measure the `0` glyph supplies it via
    /// [`with_ch_width`](Self::with_ch_width).
    #[must_use]
    pub const fn new(
        font_size: f32,
        root_font_size: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Self {
        Self {
            font_size,
            root_font_size,
            viewport_width,
            viewport_height,
            ch_width: None,
            percent_basis: None,
            environment: EnvironmentValues::none(),
        }
    }

    /// Returns a copy of this context with the host-reported `environment` set.
    ///
    /// A builder rather than a [`new`](Self::new) parameter for the same reason
    /// `ch_width` is one: only a caller in touch with the host can answer, and a
    /// context built without one stays at the no-cutout, no-keyboard default that
    /// every desktop genuinely has.
    #[must_use]
    pub const fn with_environment(mut self, environment: EnvironmentValues) -> Self {
        self.environment = environment;
        self
    }

    /// Returns a copy of this context with `percent_basis` set to `basis`.
    #[must_use]
    pub const fn with_percent_basis(mut self, basis: f32) -> Self {
        self.percent_basis = Some(basis);
        self
    }

    /// Returns a copy of this context with the measured `0`-glyph advance set to
    /// `width`, in logical pixels.
    ///
    /// Only a caller that can actually shape text can answer this, so it is a
    /// builder rather than a [`new`](Self::new) parameter: a context built
    /// without a shaper in reach stays honest about not knowing.
    #[must_use]
    pub const fn with_ch_width(mut self, width: f32) -> Self {
        self.ch_width = Some(width);
        self
    }

    /// The base a `ch` resolves against, in logical pixels.
    ///
    /// This is the measured [`ch_width`](Self::ch_width) when one is known. When
    /// it is not, CSS Values 4 §6.1.2 specifies the substitute exactly: *"in the
    /// cases where it is impossible or impractical to determine the measure of
    /// the ZERO glyph, it must be assumed to be 0.5em wide"* — so an unmeasured
    /// `ch` is `0.5 × font-size`.
    ///
    /// # Why this is a fallback and not the implementation
    ///
    /// `0.5em` is a plausible average for a proportional face and *wrong for a
    /// monospace one*, which is the case `ch` mostly exists to serve — sizing a
    /// column by character count. Resolving every `ch` to `0.5em` would look like
    /// support while silently mis-sizing the one thing anyone writes `ch` for.
    /// The spec permits the substitute only when the measure is unavailable, so
    /// it is reached only when a caller could not measure the glyph, never as a
    /// shortcut around measuring it.
    #[must_use]
    pub fn ch_base(&self) -> f32 {
        self.ch_width.unwrap_or(0.5 * self.font_size)
    }

    /// The smaller viewport dimension, in logical pixels (base for `vmin`).
    #[must_use]
    pub fn viewport_min(&self) -> f32 {
        self.viewport_width.min(self.viewport_height)
    }

    /// The larger viewport dimension, in logical pixels (base for `vmax`).
    #[must_use]
    pub fn viewport_max(&self) -> f32 {
        self.viewport_width.max(self.viewport_height)
    }

    /// The base a `cqw` resolves against, in logical pixels.
    ///
    /// css-contain-3 §6: *"The query container for each axis is the nearest
    /// ancestor container that accepts container size queries on that axis. If
    /// no eligible query container is available, then use the small viewport
    /// size for that axis."*
    ///
    /// # What this engine can and cannot answer
    ///
    /// **No element here ever accepts container size queries.** There is no
    /// `container-type` property (see `docs/conformance-audit.md`, item 4-B:
    /// `@container` is parsed, fails closed, and is inert, because a container's
    /// size is a layout *output* while style matching runs strictly before
    /// layout). So the "no eligible query container" branch is not an edge case
    /// here — it is the **only** branch, and every container unit resolves
    /// against the viewport. The *resolved-against-a-real-container* half of
    /// css-contain-3 §6 is not implemented and is not reachable; only the
    /// fallback is.
    ///
    /// The fallback base is [`viewport_width`](Self::viewport_width). This
    /// engine models a single viewport size rather than css-values-4's separate
    /// small/large/dynamic viewport sizes, so that one size stands in for the
    /// small viewport.
    #[must_use]
    pub const fn container_width(&self) -> f32 {
        self.viewport_width
    }

    /// The base a `cqh` resolves against, in logical pixels — the small viewport
    /// height. See [`container_width`](Self::container_width) for why the
    /// fallback is always taken.
    #[must_use]
    pub const fn container_height(&self) -> f32 {
        self.viewport_height
    }

    /// The base a `cqi` resolves against, in logical pixels — the small viewport
    /// **inline** size.
    ///
    /// The inline axis is horizontal here: this context carries no writing mode,
    /// as `vw`/`vh` do not need one, so `cqi` maps to the width. In a
    /// `vertical-rl` / `vertical-lr` writing mode that is wrong; the axis
    /// mapping is not implemented. See
    /// [`container_width`](Self::container_width) for why the fallback is
    /// always taken.
    #[must_use]
    pub const fn container_inline(&self) -> f32 {
        self.viewport_width
    }

    /// The base a `cqb` resolves against, in logical pixels — the small viewport
    /// **block** size, taken as the height (see
    /// [`container_inline`](Self::container_inline) on the axis mapping).
    #[must_use]
    pub const fn container_block(&self) -> f32 {
        self.viewport_height
    }

    /// The base a `cqmin` resolves against: the smaller of the `cqi` and `cqb`
    /// bases (css-contain-3 §6).
    #[must_use]
    pub fn container_min(&self) -> f32 {
        self.container_inline().min(self.container_block())
    }

    /// The base a `cqmax` resolves against: the larger of the `cqi` and `cqb`
    /// bases (css-contain-3 §6).
    #[must_use]
    pub fn container_max(&self) -> f32 {
        self.container_inline().max(self.container_block())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_leaves_percent_basis_unknown() {
        let ctx = LengthContext::new(16.0, 20.0, 800.0, 600.0);
        assert_eq!(ctx.percent_basis, None);
        assert_eq!(ctx.font_size, 16.0);
        assert_eq!(ctx.root_font_size, 20.0);
    }

    #[test]
    fn a_measured_ch_width_is_the_base_verbatim() {
        // The real path: a caller that can shape supplied the `0` advance, so
        // `ch` resolves against THAT and the 0.5em substitute never runs.
        let ctx = LengthContext::new(16.0, 16.0, 800.0, 600.0).with_ch_width(9.6);
        assert_eq!(ctx.ch_width, Some(9.6));
        assert_eq!(ctx.ch_base(), 9.6);
    }

    #[test]
    fn an_unmeasured_ch_falls_back_to_half_an_em() {
        // CSS Values 4 §6.1.2's substitute, and ONLY when unmeasured.
        let ctx = LengthContext::new(16.0, 20.0, 800.0, 600.0);
        assert_eq!(ctx.ch_width, None, "new() does not invent a measurement");
        assert_eq!(ctx.ch_base(), 8.0, "0.5 x the element's own font-size");

        // It follows `font-size`, not `root_font_size` — `ch` is an em-relative
        // substitute, so a 40px element gets 20 even though the root is 20.
        let bigger = LengthContext::new(40.0, 20.0, 800.0, 600.0);
        assert_eq!(bigger.ch_base(), 20.0);
    }

    #[test]
    fn a_measured_ch_that_disagrees_with_half_an_em_still_wins() {
        // Guards the whole point of measuring: a monospace `0` is nowhere near
        // 0.5em, and the measurement must not be quietly rounded back to it.
        let ctx = LengthContext::new(16.0, 16.0, 800.0, 600.0).with_ch_width(16.0);
        assert_ne!(ctx.ch_base(), 8.0);
        assert_eq!(ctx.ch_base(), 16.0);
    }

    #[test]
    fn ch_width_and_percent_basis_are_independent() {
        let ctx = LengthContext::new(16.0, 16.0, 800.0, 600.0)
            .with_ch_width(9.0)
            .with_percent_basis(200.0);
        assert_eq!(ctx.ch_base(), 9.0);
        assert_eq!(ctx.percent_basis, Some(200.0));
    }

    #[test]
    fn with_percent_basis_sets_it() {
        let ctx = LengthContext::new(16.0, 16.0, 800.0, 600.0).with_percent_basis(200.0);
        assert_eq!(ctx.percent_basis, Some(200.0));
    }

    #[test]
    fn viewport_min_and_max_pick_dimensions() {
        let ctx = LengthContext::new(16.0, 16.0, 800.0, 600.0);
        assert_eq!(ctx.viewport_min(), 600.0);
        assert_eq!(ctx.viewport_max(), 800.0);
    }
}
