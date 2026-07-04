//! The resolution context a relative length is evaluated against.
//!
//! Font-relative units (`em` / `rem`), viewport units (`vw` / `vh` / `vmin` /
//! `vmax`) and `calc()` / `min()` / `max()` / `clamp()` expressions carry no
//! absolute value on their own: they only become a concrete number once a
//! [`LengthContext`] supplies the bases they resolve against (spec §4.5). This
//! is the "resolve at use time" model — the symbolic length is kept until the
//! layout/compute step, then [`Length::resolve_px`](crate::property::Length::resolve_px)
//! (and [`LengthExpr::resolve`](crate::property::LengthExpr::resolve)) fold it to
//! logical pixels against this context.

/// The bases a relative length resolves against.
///
/// * `font_size` — the element's own computed `font-size`, in logical pixels;
///   the base for `em`.
/// * `root_font_size` — the document root's computed `font-size`, in logical
///   pixels; the base for `rem`.
/// * `viewport_width` / `viewport_height` — the viewport dimensions in logical
///   pixels; the bases for `vw` / `vh` (and, combined, `vmin` / `vmax`).
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
    /// The basis a `%` resolves against, if known at resolution time.
    pub percent_basis: Option<f32>,
}

impl LengthContext {
    /// Constructs a context with an unknown percentage basis.
    ///
    /// Font-relative and viewport-relative units resolve; a `%` term does not
    /// (it is resolved later, by layout, against the containing block).
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
            percent_basis: None,
        }
    }

    /// Returns a copy of this context with `percent_basis` set to `basis`.
    #[must_use]
    pub const fn with_percent_basis(mut self, basis: f32) -> Self {
        self.percent_basis = Some(basis);
        self
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
