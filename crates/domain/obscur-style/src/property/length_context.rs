#![doc = include_str!("../../docs/api/property/length_context.doc/_module.md")]

use super::EnvironmentValues;

#[doc = include_str!("../../docs/api/property/length_context.doc/LengthContext.md")]
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
    query_unit_dimensions: [Option<f32>; 4],
    vertical: bool,
}

impl LengthContext {
    #[doc = include_str!("../../docs/api/property/length_context.doc/new.md")]
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
            query_unit_dimensions: [None; 4],
            vertical: false,
        }
    }

    #[doc = include_str!("../../docs/api/property/length_context.doc/with_environment.md")]
    #[must_use]
    pub const fn with_environment(mut self, environment: EnvironmentValues) -> Self {
        self.environment = environment;
        self
    }

    pub(crate) const fn with_query_unit_dimensions(
        mut self,
        dimensions: [Option<f32>; 4],
        vertical: bool,
    ) -> Self {
        self.query_unit_dimensions = dimensions;
        self.vertical = vertical;
        self
    }

    /// Returns a copy of this context with `percent_basis` set to `basis`.
    #[must_use]
    pub const fn with_percent_basis(mut self, basis: f32) -> Self {
        self.percent_basis = Some(basis);
        self
    }

    #[doc = include_str!("../../docs/api/property/length_context.doc/with_ch_width.md")]
    #[must_use]
    pub const fn with_ch_width(mut self, width: f32) -> Self {
        self.ch_width = Some(width);
        self
    }

    #[doc = include_str!("../../docs/api/property/length_context.doc/ch_base.md")]
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

    /// The nearest eligible query container’s width size in logical pixels. If there is no eligible ancestor for this axis, use the viewport width. The engine’s logical viewport also serves as the small viewport.
    #[must_use]
    pub const fn container_width(&self) -> f32 {
        match self.query_unit_dimensions[0] {
            Some(size) => size,
            None => self.viewport_width,
        }
    }

    /// The nearest eligible query container’s height size in logical pixels. If there is no eligible ancestor for this axis, use the viewport height. The engine’s logical viewport also serves as the small viewport.
    #[must_use]
    pub const fn container_height(&self) -> f32 {
        match self.query_unit_dimensions[1] {
            Some(size) => size,
            None => self.viewport_height,
        }
    }

    /// The nearest eligible query container’s inline size in logical pixels. If there is no eligible ancestor for this axis, use the viewport inline size in the subject’s writing mode. The engine’s logical viewport also serves as the small viewport.
    #[must_use]
    pub const fn container_inline(&self) -> f32 {
        match self.query_unit_dimensions[2] {
            Some(size) => size,
            None => {
                if self.vertical {
                    self.viewport_height
                } else {
                    self.viewport_width
                }
            }
        }
    }

    /// The nearest eligible query container’s block size in logical pixels. If there is no eligible ancestor for this axis, use the viewport block size in the subject’s writing mode. The engine’s logical viewport also serves as the small viewport.
    #[must_use]
    pub const fn container_block(&self) -> f32 {
        match self.query_unit_dimensions[3] {
            Some(size) => size,
            None => {
                if self.vertical {
                    self.viewport_width
                } else {
                    self.viewport_height
                }
            }
        }
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
