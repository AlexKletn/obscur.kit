//! CSS length values.
//!
//! [`Length`] models a single CSS length term: absolute pixels, a percentage
//! (of some base resolved later during layout), the `auto` keyword, and the
//! font-relative (`em`, `rem`, `ch`) and viewport-relative (`vw`, `vh`, `vmin`,
//! `vmax`) units (spec §4.5). The relative units carry no absolute value on
//! their own; a [`LengthContext`] supplies the bases they fold against.
//! `calc()` / `min()` / `max()` / `clamp()` combine several such terms and are
//! modelled by [`LengthExpr`](crate::property::LengthExpr).
//!
//! # Two folds, for two different jobs
//!
//! * [`Length::absolutize`] runs **once**, at the computed-value step, and is
//!   what CSS means by "the computed value of a `<length>` is absolute": it turns
//!   the font- and viewport-relative units into `Px` and leaves everything that
//!   genuinely cannot be absolute yet (`%`, `auto`, an unmeasured `ch`) alone. A
//!   consumer downstream of that step reads `Px` and needs no context.
//! * [`Length::resolve_px`] folds a length **all the way to a number**, including
//!   a `%` when the caller states its basis. That is a *use*-time operation: only
//!   layout knows the containing block a percentage resolves against.

use crate::property::LengthContext;
use obscur_geometry::LogicalPx;

/// A CSS length value.
///
/// Absolute (`Px`) and percentage (`Percent`) terms are self-contained; the
/// font-relative (`Em` / `Rem` / `Ch`) and viewport-relative (`Vw` / `Vh` /
/// `Vmin` / `Vmax`) terms carry their coefficient and only become a concrete
/// number once resolved against a [`LengthContext`] via [`Length::resolve_px`].
///
/// Because the variants carry `f32` data, this type derives [`PartialEq`] but
/// not [`Eq`]/[`Hash`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    /// An absolute length in logical pixels.
    Px(LogicalPx),
    /// A percentage of some base dimension, expressed as a percentage value
    /// (for example `50.0` for `50%`). The base is resolved later, during
    /// layout, because it depends on context.
    Percent(f32),
    /// The `auto` keyword; its used value is resolved later, during layout.
    Auto,
    /// A font-relative `em` length: this many times the element's own
    /// `font-size` (`LengthContext::font_size`).
    Em(f32),
    /// A font-relative `rem` length: this many times the document root's
    /// `font-size` (`LengthContext::root_font_size`).
    Rem(f32),
    /// A font-relative `ch` length: this many times the advance of the `0`
    /// (ZERO, U+0030) glyph in the element's own font
    /// ([`LengthContext::ch_base`]).
    ///
    /// Unlike `em`, this depends on the *face*, not just the size: it is how a
    /// column is sized by character count (`width: 60ch`), which is exact in a
    /// monospace face and approximate in a proportional one. See
    /// [`LengthContext::ch_base`] for what happens when the glyph cannot be
    /// measured.
    Ch(f32),
    /// A viewport-relative `vw` length: this many percent of the viewport width.
    Vw(f32),
    /// A viewport-relative `vh` length: this many percent of the viewport
    /// height.
    Vh(f32),
    /// A viewport-relative `vmin` length: this many percent of the smaller
    /// viewport dimension.
    Vmin(f32),
    /// A viewport-relative `vmax` length: this many percent of the larger
    /// viewport dimension.
    Vmax(f32),
    /// A container-relative `cqw` length: this many percent of the query
    /// container's width. See [`LengthContext::container_width`] for what it
    /// resolves against here.
    ///
    /// [`LengthContext::container_width`]: crate::property::LengthContext::container_width
    Cqw(f32),
    /// A container-relative `cqh` length: this many percent of the query
    /// container's height.
    Cqh(f32),
    /// A container-relative `cqi` length: this many percent of the query
    /// container's inline size.
    Cqi(f32),
    /// A container-relative `cqb` length: this many percent of the query
    /// container's block size.
    Cqb(f32),
    /// A container-relative `cqmin` length: this many percent of the smaller of
    /// the `cqi` and `cqb` bases.
    Cqmin(f32),
    /// A container-relative `cqmax` length: this many percent of the larger of
    /// the `cqi` and `cqb` bases.
    Cqmax(f32),
}

impl Length {
    /// Convenience constructor for an absolute pixel length.
    #[must_use]
    pub const fn px(value: f32) -> Self {
        Self::Px(LogicalPx(value))
    }

    /// Returns the absolute pixel length if this is a [`Length::Px`].
    #[must_use]
    pub const fn as_px(self) -> Option<LogicalPx> {
        match self {
            Self::Px(px) => Some(px),
            _ => None,
        }
    }

    /// Returns `true` when this length is the `auto` keyword.
    #[must_use]
    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }

    /// Returns `true` when this length is a `ch` term.
    ///
    /// Every other unit resolves from numbers a caller already holds; `ch` alone
    /// needs the `0` glyph *shaped* to answer. This predicate lets a caller find
    /// out whether that measurement is worth taking before taking it — see
    /// [`LengthContext::ch_base`](crate::property::LengthContext::ch_base).
    #[must_use]
    pub const fn is_ch(self) -> bool {
        matches!(self, Self::Ch(_))
    }

    /// Folds this length's **absolutizable** units to [`Length::Px`], leaving
    /// every other term symbolic (CSS Values 4 §5: the computed value of a
    /// `<length>` is absolute).
    ///
    /// `em` / `rem` / `vw` / `vh` / `vmin` / `vmax` all resolve from numbers a
    /// computed-value pass already holds, so they fold here and no downstream
    /// consumer needs a [`LengthContext`] to read them. Three terms do not fold:
    ///
    /// * `Percent` resolves against a containing block that layout has not
    ///   produced yet, so it stays a percentage. (`ctx.percent_basis` is ignored
    ///   deliberately: a caller that knows the basis — `font-size: 150%` against
    ///   the parent's size — folds it with [`resolve_px`](Self::resolve_px)
    ///   instead, where the basis is a stated input rather than a silent one.)
    /// * `Auto` has no numeric value at all.
    /// * `Ch` folds **only** when `ctx` carries a measured `ch_width`. Without
    ///   one, folding it here would bake in [`LengthContext::ch_base`]'s `0.5em`
    ///   substitute and permanently mis-size the monospace column `ch` exists
    ///   for; leaving it symbolic routes it to a consumer that *can* shape the
    ///   `0` glyph.
    #[must_use]
    pub fn absolutize(self, ctx: &LengthContext) -> Self {
        match self {
            Self::Px(_) | Self::Percent(_) | Self::Auto => self,
            Self::Ch(_) if ctx.ch_width.is_none() => self,
            relative => relative.resolve_px(ctx).map_or(self, Self::px),
        }
    }

    /// Resolves this length to concrete logical pixels against `ctx`.
    ///
    /// * `Px` returns its value directly; the font- and viewport-relative units
    ///   multiply their coefficient by the matching base from `ctx` — `Ch`
    ///   against [`LengthContext::ch_base`], which substitutes `0.5em` when the
    ///   `0` glyph was not measurable.
    /// * `Percent` resolves only when `ctx.percent_basis` is known (it usually
    ///   is not here — bare percentages are resolved later by layout against the
    ///   containing block), otherwise it returns [`None`].
    /// * `Auto` has no numeric value and returns [`None`].
    #[must_use]
    pub fn resolve_px(self, ctx: &LengthContext) -> Option<f32> {
        match self {
            Self::Px(px) => Some(px.0),
            Self::Em(factor) => Some(factor * ctx.font_size),
            Self::Rem(factor) => Some(factor * ctx.root_font_size),
            Self::Ch(factor) => Some(factor * ctx.ch_base()),
            Self::Vw(pct) => Some(pct / 100.0 * ctx.viewport_width),
            Self::Vh(pct) => Some(pct / 100.0 * ctx.viewport_height),
            Self::Vmin(pct) => Some(pct / 100.0 * ctx.viewport_min()),
            Self::Vmax(pct) => Some(pct / 100.0 * ctx.viewport_max()),
            Self::Cqw(pct) => Some(pct / 100.0 * ctx.container_width()),
            Self::Cqh(pct) => Some(pct / 100.0 * ctx.container_height()),
            Self::Cqi(pct) => Some(pct / 100.0 * ctx.container_inline()),
            Self::Cqb(pct) => Some(pct / 100.0 * ctx.container_block()),
            Self::Cqmin(pct) => Some(pct / 100.0 * ctx.container_min()),
            Self::Cqmax(pct) => Some(pct / 100.0 * ctx.container_max()),
            Self::Percent(pct) => ctx.percent_basis.map(|basis| pct / 100.0 * basis),
            Self::Auto => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn px_constructor_wraps_logical_px() {
        assert_eq!(Length::px(12.0), Length::Px(LogicalPx(12.0)));
    }

    #[test]
    fn as_px_extracts_only_pixels() {
        assert_eq!(Length::px(4.0).as_px(), Some(LogicalPx(4.0)));
        assert_eq!(Length::Percent(50.0).as_px(), None);
        assert_eq!(Length::Auto.as_px(), None);
    }

    #[test]
    fn is_auto_detects_auto() {
        assert!(Length::Auto.is_auto());
        assert!(!Length::px(1.0).is_auto());
    }

    fn ctx() -> LengthContext {
        // font 16, root 20 (distinct so `em` vs `rem` cannot be confused),
        // viewport 800x600.
        LengthContext::new(16.0, 20.0, 800.0, 600.0)
    }

    #[test]
    fn absolutize_folds_every_font_and_viewport_unit_to_px() {
        let ctx = ctx();
        assert_eq!(Length::Em(2.0).absolutize(&ctx), Length::px(32.0));
        assert_eq!(Length::Rem(2.0).absolutize(&ctx), Length::px(40.0));
        assert_eq!(Length::Vw(5.0).absolutize(&ctx), Length::px(40.0));
        assert_eq!(Length::Vh(10.0).absolutize(&ctx), Length::px(60.0));
        assert_eq!(Length::Vmin(10.0).absolutize(&ctx), Length::px(60.0));
        assert_eq!(Length::Vmax(10.0).absolutize(&ctx), Length::px(80.0));
    }

    #[test]
    fn absolutize_is_idempotent_and_leaves_px_alone() {
        let ctx = ctx();
        let once = Length::Em(2.0).absolutize(&ctx);
        // An inherited computed value is absolutized again by the child's own
        // pass; that second pass must not re-scale it against the child's `em`.
        assert_eq!(once.absolutize(&ctx), once);
        assert_eq!(Length::px(7.0).absolutize(&ctx), Length::px(7.0));
    }

    #[test]
    fn absolutize_keeps_percent_and_auto_symbolic() {
        // Neither has an absolute value at computed-value time: `%` needs the
        // containing block, `auto` needs the used value.
        let ctx = ctx();
        assert_eq!(
            Length::Percent(50.0).absolutize(&ctx),
            Length::Percent(50.0)
        );
        assert_eq!(Length::Auto.absolutize(&ctx), Length::Auto);
        // Even a KNOWN basis does not tempt it: folding `%` here would strip the
        // symbol layout needs against a *different* basis.
        let with_basis = ctx.with_percent_basis(200.0);
        assert_eq!(
            Length::Percent(50.0).absolutize(&with_basis),
            Length::Percent(50.0)
        );
    }

    #[test]
    fn absolutize_folds_ch_only_when_the_zero_glyph_was_measured() {
        // Unmeasured: staying symbolic keeps the 0.5em substitute out of the
        // computed value, so a consumer that can shape still gets the real one.
        assert_eq!(Length::Ch(20.0).absolutize(&ctx()), Length::Ch(20.0));
        // Measured: it folds like any other font-relative unit.
        assert_eq!(
            Length::Ch(20.0).absolutize(&ctx().with_ch_width(9.6)),
            Length::px(192.0)
        );
    }

    #[test]
    fn em_resolves_against_font_size() {
        assert_eq!(Length::Em(3.0).resolve_px(&ctx()), Some(48.0));
    }

    #[test]
    fn rem_resolves_against_root_font_size() {
        // Root font size (20), not the element font size (16).
        assert_eq!(Length::Rem(2.0).resolve_px(&ctx()), Some(40.0));
    }

    #[test]
    fn ch_resolves_against_the_measured_zero_advance() {
        // The unit's reason to exist: `20ch` is twenty characters wide in a face
        // whose `0` advances 9.6px — not twenty half-ems.
        let ctx = ctx().with_ch_width(9.6);
        assert_eq!(Length::Ch(20.0).resolve_px(&ctx), Some(192.0));
    }

    #[test]
    fn an_unmeasured_ch_resolves_through_the_half_em_substitute() {
        // No measurement available: §6.1.2's 0.5em. Element font-size 16 → 8/ch.
        assert_eq!(Length::Ch(3.0).resolve_px(&ctx()), Some(24.0));
    }

    #[test]
    fn ch_is_not_em_when_the_face_is_measured() {
        // The two units must not be confused: a monospace `0` advancing a full
        // em apart, `ch` follows the FACE while `em` follows the size.
        let narrow = ctx().with_ch_width(8.0);
        let wide = ctx().with_ch_width(16.0);
        assert_eq!(
            Length::Em(1.0).resolve_px(&narrow),
            Length::Em(1.0).resolve_px(&wide)
        );
        assert_ne!(
            Length::Ch(1.0).resolve_px(&narrow),
            Length::Ch(1.0).resolve_px(&wide)
        );
    }

    #[test]
    fn viewport_units_resolve_against_viewport() {
        assert_eq!(Length::Vw(5.0).resolve_px(&ctx()), Some(40.0)); // 5% of 800
        assert_eq!(Length::Vh(10.0).resolve_px(&ctx()), Some(60.0)); // 10% of 600
        assert_eq!(Length::Vmin(10.0).resolve_px(&ctx()), Some(60.0)); // 10% of 600
        assert_eq!(Length::Vmax(10.0).resolve_px(&ctx()), Some(80.0)); // 10% of 800
    }

    /// css-contain-3 §6: with no eligible query container, each container unit
    /// falls back to the small viewport size *for that axis* — so `cqw`/`cqi`
    /// track the width and `cqh`/`cqb` the height, and `cqmin`/`cqmax` are the
    /// smaller/larger of the inline and block bases.
    #[test]
    fn container_units_fall_back_to_the_viewport_per_axis() {
        // ctx() is 800x600.
        assert_eq!(Length::Cqw(50.0).resolve_px(&ctx()), Some(400.0));
        assert_eq!(Length::Cqh(50.0).resolve_px(&ctx()), Some(300.0));
        assert_eq!(Length::Cqi(50.0).resolve_px(&ctx()), Some(400.0));
        assert_eq!(Length::Cqb(50.0).resolve_px(&ctx()), Some(300.0));
        assert_eq!(Length::Cqmin(10.0).resolve_px(&ctx()), Some(60.0));
        assert_eq!(Length::Cqmax(10.0).resolve_px(&ctx()), Some(80.0));
    }

    /// Container units carry no basis a later pass could improve on (no element
    /// in this engine accepts container size queries — see
    /// `LengthContext::container_width`), so they retire at the computed-value
    /// step like the viewport units rather than staying symbolic.
    #[test]
    fn container_units_absolutize_at_the_computed_value_step() {
        assert_eq!(Length::Cqi(50.0).absolutize(&ctx()), Length::px(400.0));
        assert_eq!(Length::Cqb(50.0).absolutize(&ctx()), Length::px(300.0));
    }

    #[test]
    fn px_resolves_to_itself_percent_needs_basis() {
        assert_eq!(Length::px(12.0).resolve_px(&ctx()), Some(12.0));
        // No basis: a bare percentage does not fold here (layout resolves it).
        assert_eq!(Length::Percent(50.0).resolve_px(&ctx()), None);
        // With a basis it folds.
        assert_eq!(
            Length::Percent(50.0).resolve_px(&ctx().with_percent_basis(200.0)),
            Some(100.0)
        );
        assert_eq!(Length::Auto.resolve_px(&ctx()), None);
    }
}
