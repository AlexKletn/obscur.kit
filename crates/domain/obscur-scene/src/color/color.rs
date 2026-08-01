#![doc = include_str!("color.doc/_module.md")]

#[doc = include_str!("color.doc/Color.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    #[doc = include_str!("color.doc/r.md")]
    pub r: f32,
    #[doc = include_str!("color.doc/g.md")]
    pub g: f32,
    #[doc = include_str!("color.doc/b.md")]
    pub b: f32,
    #[doc = include_str!("color.doc/a.md")]
    pub a: f32,
}

impl Color {
    #[doc = include_str!("color.doc/TRANSPARENT.md")]
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    #[doc = include_str!("color.doc/BLACK.md")]
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    #[doc = include_str!("color.doc/WHITE.md")]
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    #[doc = include_str!("color.doc/rgba.md")]
    #[must_use]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    #[doc = include_str!("color.doc/rgb.md")]
    #[must_use]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    #[doc = include_str!("color.doc/with_alpha.md")]
    #[must_use]
    pub const fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    #[doc = include_str!("color.doc/is_opaque.md")]
    #[must_use]
    pub fn is_opaque(self) -> bool {
        self.a >= 1.0
    }

    #[doc = include_str!("color.doc/is_transparent.md")]
    #[must_use]
    pub fn is_transparent(self) -> bool {
        self.a <= 0.0
    }

    #[doc = include_str!("color.doc/premultiply.md")]
    #[must_use]
    pub fn premultiply(self) -> Color {
        Self {
            r: self.r * self.a,
            g: self.g * self.a,
            b: self.b * self.a,
            a: self.a,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_is_opaque() {
        let c = Color::rgb(0.2, 0.4, 0.6);
        assert_eq!(c, Color::rgba(0.2, 0.4, 0.6, 1.0));
        assert!(c.is_opaque());
        assert!(!c.is_transparent());
    }

    #[test]
    fn with_alpha_replaces_alpha() {
        let c = Color::WHITE.with_alpha(0.5);
        assert_eq!(c, Color::rgba(1.0, 1.0, 1.0, 0.5));
    }

    #[test]
    fn transparency_predicates() {
        assert!(Color::TRANSPARENT.is_transparent());
        assert!(!Color::TRANSPARENT.is_opaque());
        assert!(Color::BLACK.is_opaque());
    }

    #[test]
    fn premultiply_scales_channels() {
        let c = Color::rgba(1.0, 0.5, 0.25, 0.5);
        assert_eq!(c.premultiply(), Color::rgba(0.5, 0.25, 0.125, 0.5));
    }

    #[test]
    fn default_is_transparent_black() {
        assert_eq!(Color::default(), Color::TRANSPARENT);
    }
}
