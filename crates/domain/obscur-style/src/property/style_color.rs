#![doc = include_str!("style_color.doc/_module.md")]

#[doc = include_str!("style_color.doc/StyleColor.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StyleColor {
    #[doc = include_str!("style_color.doc/r.md")]
    pub r: f32,
    #[doc = include_str!("style_color.doc/g.md")]
    pub g: f32,
    #[doc = include_str!("style_color.doc/b.md")]
    pub b: f32,
    #[doc = include_str!("style_color.doc/a.md")]
    pub a: f32,
}

impl StyleColor {
    #[doc = include_str!("style_color.doc/TRANSPARENT.md")]
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    #[doc = include_str!("style_color.doc/BLACK.md")]
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    #[doc = include_str!("style_color.doc/rgba.md")]
    #[must_use]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_stores_channels() {
        let c = StyleColor::rgba(0.25, 0.5, 0.75, 1.0);
        assert_eq!(c.r, 0.25);
        assert_eq!(c.g, 0.5);
        assert_eq!(c.b, 0.75);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn transparent_is_zero_alpha() {
        assert_eq!(StyleColor::TRANSPARENT.a, 0.0);
    }

    #[test]
    fn black_is_opaque() {
        assert_eq!(StyleColor::BLACK, StyleColor::rgba(0.0, 0.0, 0.0, 1.0));
    }
}
