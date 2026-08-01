#![doc = include_str!("layout_snapshot.doc/_module.md")]

use crate::model::LayoutBox;
use obscur_identity::VersionStamp;

#[doc = include_str!("layout_snapshot.doc/LayoutSnapshot.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutSnapshot {
    layout_box: LayoutBox,
    version: VersionStamp,
}

impl LayoutSnapshot {
    #[doc = include_str!("layout_snapshot.doc/new.md")]
    #[must_use]
    pub const fn new(layout_box: LayoutBox, version: VersionStamp) -> Self {
        Self {
            layout_box,
            version,
        }
    }

    #[doc = include_str!("layout_snapshot.doc/layout_box.md")]
    #[must_use]
    pub const fn layout_box(self) -> LayoutBox {
        self.layout_box
    }

    #[doc = include_str!("layout_snapshot.doc/version.md")]
    #[must_use]
    pub const fn version(self) -> VersionStamp {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutSnapshot;
    use crate::model::LayoutBox;
    use obscur_geometry::Size;
    use obscur_identity::VersionStamp;

    #[test]
    fn accessors_round_trip() {
        let layout_box = LayoutBox::from_size(Size::new(50.0, 20.0));
        let version = VersionStamp::from_raw(7);
        let snapshot = LayoutSnapshot::new(layout_box, version);
        assert_eq!(snapshot.layout_box(), layout_box);
        assert_eq!(snapshot.version(), version);
    }
}
