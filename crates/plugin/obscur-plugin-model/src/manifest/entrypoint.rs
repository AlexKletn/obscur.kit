//! Entrypoint (skeleton stub for the spec-defined entity; not yet implemented).

/// A package entry that an executor may load after validation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entrypoint {
    /// The package-relative path of the declared executable or module.
    pub path: Box<str>,
}
