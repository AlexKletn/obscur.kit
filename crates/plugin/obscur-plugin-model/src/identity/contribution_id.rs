//! `ContributionId` (skeleton stub for the spec-defined entity; not yet implemented).

/// A validated plugin-local contribution identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContributionId(Box<str>);

impl ContributionId {
    /// Accepts a nonempty ASCII identifier with letters, digits, dots, underscores, or hyphens. The first character must be alphanumeric.
    #[must_use]
    pub fn new(value: &str) -> Option<Self> {
        super::local_id::valid_local_id(value).then(|| Self(value.into()))
    }

    /// The original case-sensitive spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
