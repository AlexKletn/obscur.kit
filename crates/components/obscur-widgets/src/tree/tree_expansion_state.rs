//! Tree expansion snapshots for application-owned storage.

use serde::{Deserialize, Serialize};

/// A versioned snapshot containing the stable IDs of expanded branches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeExpansionState {
    version: u32,
    expanded_ids: Vec<Box<str>>,
}

impl TreeExpansionState {
    const VERSION: u32 = 1;

    pub(super) fn capture(expanded_ids: Vec<Box<str>>) -> Self {
        Self {
            version: Self::VERSION,
            expanded_ids,
        }
    }

    pub(super) fn ids(&self) -> Option<&[Box<str>]> {
        (self.version == Self::VERSION).then_some(&self.expanded_ids)
    }
}
