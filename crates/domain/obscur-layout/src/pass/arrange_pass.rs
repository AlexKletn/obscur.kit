#![doc = include_str!("arrange_pass.doc/_module.md")]

use obscur_geometry::{Point, Rect, Size};

#[doc = include_str!("arrange_pass.doc/ArrangePass.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrangePass {
    #[doc = include_str!("arrange_pass.doc/region.md")]
    region: Rect,
}

impl ArrangePass {
    #[doc = include_str!("arrange_pass.doc/new.md")]
    #[must_use]
    pub const fn new(region: Rect) -> Self {
        Self { region }
    }

    #[doc = include_str!("arrange_pass.doc/region_2.md")]
    #[must_use]
    pub const fn region(self) -> Rect {
        self.region
    }

    #[doc = include_str!("arrange_pass.doc/size.md")]
    #[must_use]
    pub const fn size(self) -> Size {
        self.region.size
    }

    #[doc = include_str!("arrange_pass.doc/origin.md")]
    #[must_use]
    pub const fn origin(self) -> Point {
        self.region.origin
    }
}

#[cfg(test)]
mod tests {
    use super::ArrangePass;
    use obscur_geometry::{Point, Rect, Size};

    #[test]
    fn exposes_region_size_and_origin() {
        let region = Rect::new(Point::new(4.0, 5.0), Size::new(20.0, 10.0));
        let pass = ArrangePass::new(region);
        assert_eq!(pass.region(), region);
        assert_eq!(pass.size(), Size::new(20.0, 10.0));
        assert_eq!(pass.origin(), Point::new(4.0, 5.0));
    }
}
