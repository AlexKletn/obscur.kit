use crate::identity::ContributionId;

/// An application-defined identifier for a plugin integration point.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtensionPoint(ContributionId);

impl ExtensionPoint {
    /// Parses an identifier chosen by the host application.
    #[must_use]
    pub fn new(id: &str) -> Option<Self> {
        ContributionId::new(id).map(Self)
    }

    /// Returns the application-defined identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        self.0.as_str()
    }
}
