//! Serializable focus state for application-owned persistence.

use serde::{Deserialize, Serialize};

/// A versioned reference to a focus target by its authored automation ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusState {
    version: u32,
    automation_id: Box<str>,
}

impl FocusState {
    const VERSION: u32 = 1;

    pub(super) fn capture(automation_id: &str) -> Self {
        Self {
            version: Self::VERSION,
            automation_id: automation_id.into(),
        }
    }

    pub(super) fn automation_id(&self) -> Option<&str> {
        (self.version == Self::VERSION && !self.automation_id.trim().is_empty())
            .then_some(&self.automation_id)
    }
}
