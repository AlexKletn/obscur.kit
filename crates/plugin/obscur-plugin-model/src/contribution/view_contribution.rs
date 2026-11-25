//! `ViewContribution` (skeleton stub for the spec-defined entity; not yet implemented).

use crate::{identity::ContributionId, manifest::VersionReq};

/// The plugin-owned definition of a view. The host assigns its placement and route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewContribution {
    id: ContributionId,
    title: Box<str>,
    order: i32,
    contract: Option<VersionReq>,
}

impl ViewContribution {
    /// Defines a page with a plugin-local ID and title.
    #[must_use]
    pub fn new(id: ContributionId, title: impl Into<Box<str>>) -> Self {
        Self {
            id,
            title: title.into(),
            order: 0,
            contract: None,
        }
    }

    /// Sets the requested order within settings. Lower values appear first.
    #[must_use]
    pub const fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    /// Declares the required version range of the application extension point.
    #[must_use]
    pub fn with_contract(mut self, contract: VersionReq) -> Self {
        self.contract = Some(contract);
        self
    }

    /// Returns the requested extension point version range, if declared.
    #[must_use]
    pub const fn contract(&self) -> Option<&VersionReq> {
        self.contract.as_ref()
    }

    /// The contribution key within its plugin.
    #[must_use]
    pub const fn id(&self) -> &ContributionId {
        &self.id
    }

    /// The label displayed by host-owned settings chrome.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// The requested ordering priority.
    #[must_use]
    pub const fn order(&self) -> i32 {
        self.order
    }
}
