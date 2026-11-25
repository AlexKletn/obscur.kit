//! Docking snapshots for application-owned storage.

use serde::{Deserialize, Serialize};

/// A versioned snapshot of docking panel IDs and sides in workspace order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockingState {
    version: u32,
    panels: Vec<(Box<str>, Box<str>)>,
}

impl DockingState {
    const VERSION: u32 = 1;

    pub(super) fn capture(panels: Vec<(Box<str>, Box<str>)>) -> Self {
        Self {
            version: Self::VERSION,
            panels,
        }
    }

    pub(super) fn panels(&self) -> Option<&[(Box<str>, Box<str>)]> {
        (self.version == Self::VERSION).then_some(&self.panels)
    }
}
