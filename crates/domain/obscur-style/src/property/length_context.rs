#![doc = include_str!("length_context.doc/_module.md")]

use super::EnvironmentValues;

#[doc = include_str!("length_context.doc/LengthContext.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LengthContext {
    #[doc = include_str!("length_context.doc/font_size.md")]
    pub font_size: f32,
    #[doc = include_str!("length_context.doc/root_font_size.md")]
    pub root_font_size: f32,
    #[doc = include_str!("length_context.doc/viewport_width.md")]
    pub viewport_width: f32,
    #[doc = include_str!("length_context.doc/viewport_height.md")]
    pub viewport_height: f32,
    #[doc = include_str!("length_context.doc/ch_width.md")]
    pub ch_width: Option<f32>,
    #[doc = include_str!("length_context.doc/percent_basis.md")]
    pub percent_basis: Option<f32>,
    #[doc = include_str!("length_context.doc/environment.md")]
    pub environment: EnvironmentValues,
}

impl LengthContext {
    #[doc = include_str!("length_context.doc/new.md")]
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

    #[doc = include_str!("length_context.doc/with_environment.md")]
    #[must_use]
    pub const fn with_environment(mut self, environment: EnvironmentValues) -> Self {
        self.environment = environment;
        self
    }

    #[doc = include_str!("length_context.doc/with_percent_basis.md")]
    #[must_use]
    pub const fn with_percent_basis(mut self, basis: f32) -> Self {
        self.percent_basis = Some(basis);
        self
    }

    #[doc = include_str!("length_context.doc/with_ch_width.md")]
    #[must_use]
    pub const fn with_ch_width(mut self, width: f32) -> Self {
        self.ch_width = Some(width);
        self
    }

    #[doc = include_str!("length_context.doc/ch_base.md")]
    #[must_use]
    pub fn ch_base(&self) -> f32 {
        self.ch_width.unwrap_or(0.5 * self.font_size)
    }

    #[doc = include_str!("length_context.doc/viewport_min.md")]
    #[must_use]
    pub fn viewport_min(&self) -> f32 {
        self.viewport_width.min(self.viewport_height)
    }

    #[doc = include_str!("length_context.doc/viewport_max.md")]
    #[must_use]
    pub fn viewport_max(&self) -> f32 {
        self.viewport_width.max(self.viewport_height)
    }

    #[doc = include_str!("length_context.doc/container_width.md")]
    #[must_use]
    pub const fn container_width(&self) -> f32 {
        self.viewport_width
    }

    #[doc = include_str!("length_context.doc/container_height.md")]
    #[must_use]
    pub const fn container_height(&self) -> f32 {
        self.viewport_height
    }

    #[doc = include_str!("length_context.doc/container_inline.md")]
    #[must_use]
    pub const fn container_inline(&self) -> f32 {
        self.viewport_width
    }

    #[doc = include_str!("length_context.doc/container_block.md")]
    #[must_use]
    pub const fn container_block(&self) -> f32 {
        self.viewport_height
    }

    #[doc = include_str!("length_context.doc/container_min.md")]
    #[must_use]
    pub fn container_min(&self) -> f32 {
        self.container_inline().min(self.container_block())
    }

    #[doc = include_str!("length_context.doc/container_max.md")]
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
