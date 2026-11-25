//! `PluginId` (skeleton stub for the spec-defined entity; not yet implemented).

/// A validated plugin identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize)]
#[serde(try_from = "String")]
pub struct PluginId(Box<str>);

impl PluginId {
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

impl TryFrom<String> for PluginId {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        super::local_id::valid_local_id(&value)
            .then(|| Self(value.into_boxed_str()))
            .ok_or("invalid plugin ID")
    }
}
