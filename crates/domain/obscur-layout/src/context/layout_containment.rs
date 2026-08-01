#![doc = include_str!("layout_containment.doc/_module.md")]

#[doc = include_str!("layout_containment.doc/LayoutContainment.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayoutContainment {
    #[doc = include_str!("layout_containment.doc/contains_layout.md")]
    contains_layout: bool,
    #[doc = include_str!("layout_containment.doc/contains_size.md")]
    contains_size: bool,
}

impl LayoutContainment {
    #[doc = include_str!("layout_containment.doc/none.md")]
    #[must_use]
    pub const fn none() -> Self {
        Self {
            contains_layout: false,
            contains_size: false,
        }
    }

    #[doc = include_str!("layout_containment.doc/layout.md")]
    #[must_use]
    pub const fn layout() -> Self {
        Self {
            contains_layout: true,
            contains_size: false,
        }
    }

    #[doc = include_str!("layout_containment.doc/size.md")]
    #[must_use]
    pub const fn size() -> Self {
        Self {
            contains_layout: false,
            contains_size: true,
        }
    }

    #[doc = include_str!("layout_containment.doc/layout_and_size.md")]
    #[must_use]
    pub const fn layout_and_size() -> Self {
        Self {
            contains_layout: true,
            contains_size: true,
        }
    }

    #[doc = include_str!("layout_containment.doc/contains_layout_2.md")]
    #[must_use]
    pub const fn contains_layout(self) -> bool {
        self.contains_layout
    }

    #[doc = include_str!("layout_containment.doc/contains_size_2.md")]
    #[must_use]
    pub const fn contains_size(self) -> bool {
        self.contains_size
    }

    #[doc = include_str!("layout_containment.doc/is_boundary.md")]
    #[must_use]
    pub const fn is_boundary(self) -> bool {
        self.contains_layout
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutContainment;

    #[test]
    fn none_contains_nothing_and_is_not_boundary() {
        let c = LayoutContainment::none();
        assert!(!c.contains_layout());
        assert!(!c.contains_size());
        assert!(!c.is_boundary());
    }

    #[test]
    fn layout_is_boundary() {
        let c = LayoutContainment::layout();
        assert!(c.contains_layout());
        assert!(!c.contains_size());
        assert!(c.is_boundary());
    }

    #[test]
    fn size_only_is_not_boundary() {
        let c = LayoutContainment::size();
        assert!(!c.contains_layout());
        assert!(c.contains_size());
        assert!(!c.is_boundary());
    }

    #[test]
    fn layout_and_size_contains_both_and_is_boundary() {
        let c = LayoutContainment::layout_and_size();
        assert!(c.contains_layout());
        assert!(c.contains_size());
        assert!(c.is_boundary());
    }
}
