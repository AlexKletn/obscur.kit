use obscur_layout::algorithm::DockSide;

#[doc = include_str!("../../docs/api/docking/docking.doc/DockPanel.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockPanel {
    pub(super) id: Box<str>,
    pub(super) title: Box<str>,
    pub(super) side: DockSide,
}

impl DockPanel {
    /// Creates a panel with stable `id`, display `title`, docked against `side`.
    #[must_use]
    pub fn new(id: impl Into<Box<str>>, title: impl Into<Box<str>>, side: DockSide) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            side,
        }
    }

    /// Returns the panel's stable id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the panel title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the edge the panel is docked against.
    #[must_use]
    pub const fn side(&self) -> DockSide {
        self.side
    }
}
