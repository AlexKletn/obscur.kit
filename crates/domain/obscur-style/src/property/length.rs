//! CSS length values.
//!
//! [`Length`] models a single CSS length term: absolute pixels, a percentage
//! (of some base resolved later during layout), the `auto` keyword, and the
//! font-relative (`em`, `rem`, `ch`) and viewport-relative (`vw`, `vh`, `vmin`,
//! `vmax`) units (spec §4.5). The relative units carry no absolute value on
//! their own; [`Length::resolve_px`] folds them to logical pixels against a
//! [`LengthContext`] at the layout/compute step ("resolve at use time").
//! `calc()` / `min()` / `max()` / `clamp()` combine several such terms and are
//! modelled by [`LengthExpr`](crate::property::LengthExpr).

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
