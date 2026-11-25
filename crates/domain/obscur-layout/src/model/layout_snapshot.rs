//! An immutable, versioned snapshot of a node's arranged layout.

use crate::model::LayoutBox;
use obscur_identity::VersionStamp;

#[doc = include_str!("../../docs/api/model/layout_snapshot.doc/LayoutSnapshot.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutSnapshot {
    layout_box: LayoutBox,
    version: VersionStamp,
}

impl LayoutSnapshot {
    /// Creates a snapshot pairing an arranged `layout_box` with its `version`.
    #[must_use]
    pub const fn new(layout_box: LayoutBox, version: VersionStamp) -> Self {
        Self {
            layout_box,
            version,
        }
    }

    /// The arranged layout box captured by this snapshot.
    #[must_use]
    pub const fn layout_box(self) -> LayoutBox {
        self.layout_box
    }

    /// The version stamp under which this snapshot was produced.
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
