//! Selected-tab snapshots for application-owned storage.

use serde::{Deserialize, Serialize};

/// A versioned snapshot identifying the selected tab by its explicit ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabSelectionState {
    version: u32,
    tab_id: Box<str>,
}

impl TabSelectionState {
    const VERSION: u32 = 1;

    pub(super) fn capture(id: &str) -> Self {
        Self {
            version: Self::VERSION,
            tab_id: id.into(),
        }
    }

    pub(super) fn validated_id(&self) -> Option<&str> {
        (self.version == Self::VERSION && !self.tab_id.trim().is_empty()).then_some(&self.tab_id)
    }
}
