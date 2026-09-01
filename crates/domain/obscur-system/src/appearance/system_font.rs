#![doc = include_str!("system_font.doc/_module.md")]

#[doc = include_str!("system_font.doc/SystemFont.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct SystemFont {
    #[doc = include_str!("system_font.doc/family.md")]
    pub family: Box<str>,
    #[doc = include_str!("system_font.doc/size_pt.md")]
    pub size_pt: Option<f32>,
}

impl SystemFont {
    #[doc = include_str!("system_font.doc/new.md")]
    #[must_use]
    pub fn new(family: impl Into<Box<str>>, size_pt: Option<f32>) -> Self {
        Self {
            family: family.into(),
            size_pt,
        }
    }

    #[doc = include_str!("system_font.doc/size_px.md")]
    #[must_use]
    pub fn size_px(&self) -> Option<f32> {
        // The CSS reference pixel is 1/96 in and a point is 1/72 in, so the
        // ratio is exact and not a rendering choice. A desktop that states a
        // size in points is stating a physical size; scaling on top of it is
        // the display's business, not this conversion's.
        self.size_pt.map(|pt| pt * 96.0 / 72.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_size_in_points_becomes_the_css_pixel_size() {
        let font = SystemFont::new("Cantarell", Some(12.0));
        assert_eq!(font.size_px(), Some(16.0));
    }

    #[test]
    fn a_family_with_no_size_stays_a_family_with_no_size() {
        // A desktop may name a font and say nothing about its size, and
        // inventing one would be worse than leaving it to the stylesheet.
        let font = SystemFont::new("Noto Sans", None);
        assert_eq!(font.size_px(), None);
        assert_eq!(&*font.family, "Noto Sans");
    }

    #[test]
    fn the_ratio_is_the_exact_one() {
        // 9pt is a common desktop default and lands on a non-integer pixel
        // size; rounding it here would be a rendering decision taken too early.
        let font = SystemFont::new("Sans", Some(9.0));
        assert_eq!(font.size_px(), Some(12.0));
        let odd = SystemFont::new("Sans", Some(11.0));
        assert!((odd.size_px().unwrap() - 14.666_667).abs() < 0.001);
    }
}
