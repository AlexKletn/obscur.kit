//! `ResolutionError`: why name resolution failed for a reference (spec §2.11).

/// The kind of failure a [`ResolutionError`] reports (spec §2.11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionErrorKind {
    /// An expression referenced an identifier that is not defined in any
    /// enclosing scope (spec §2.10).
    UnknownName,
    /// A name was defined twice in the same scope (spec §2.10).
    DuplicateDefinition,
    /// A `<For>` loop lacks the mandatory `key` attribute that gives each item
    /// a stable identity (spec §2.11).
    MissingLoopKey,
}

/// Why name resolution failed for a single reference or declaration (spec
/// §2.11).
///
/// The error carries the offending `name` and the byte `offset` into the source
/// where the failure was detected, so a diagnostic can point at it. This type
/// implements [`std::error::Error`] so it composes with the standard error
/// ecosystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionError {
    kind: ResolutionErrorKind,
    name: Box<str>,
    offset: u32,
}

impl ResolutionError {
    /// Creates a resolution error of `kind` for `name` at byte `offset`.
    #[must_use]
    pub fn new(kind: ResolutionErrorKind, name: impl Into<Box<str>>, offset: u32) -> Self {
        Self {
            kind,
            name: name.into(),
            offset,
        }
    }

    /// Creates an [`UnknownName`](ResolutionErrorKind::UnknownName) error.
    #[must_use]
    pub fn unknown_name(name: impl Into<Box<str>>, offset: u32) -> Self {
        Self::new(ResolutionErrorKind::UnknownName, name, offset)
    }

    /// Creates a [`DuplicateDefinition`](ResolutionErrorKind::DuplicateDefinition)
    /// error.
    #[must_use]
    pub fn duplicate(name: impl Into<Box<str>>, offset: u32) -> Self {
        Self::new(ResolutionErrorKind::DuplicateDefinition, name, offset)
    }

    /// Creates a [`MissingLoopKey`](ResolutionErrorKind::MissingLoopKey) error;
    /// the `key` attribute is the missing name.
    #[must_use]
    pub fn missing_loop_key(offset: u32) -> Self {
        Self::new(ResolutionErrorKind::MissingLoopKey, "key", offset)
    }

    /// Returns the error's kind.
    #[must_use]
    pub const fn kind(&self) -> &ResolutionErrorKind {
        &self.kind
    }

    /// Returns the offending name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the byte offset into the source where the failure was detected.
    #[must_use]
    pub const fn offset(&self) -> u32 {
        self.offset
    }
}

impl std::fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ResolutionErrorKind::UnknownName => {
                write!(f, "unknown name `{}` at offset {}", self.name, self.offset)
            }
            ResolutionErrorKind::DuplicateDefinition => {
                write!(
                    f,
                    "duplicate definition `{}` at offset {}",
                    self.name, self.offset
                )
            }
            ResolutionErrorKind::MissingLoopKey => {
                write!(
                    f,
                    "loop is missing a `key` attribute at offset {}",
                    self.offset
                )
            }
        }
    }
}

impl std::error::Error for ResolutionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_renders_each_kind() {
        assert_eq!(
            ResolutionError::unknown_name("missing", 4).to_string(),
            "unknown name `missing` at offset 4"
        );
        assert_eq!(
            ResolutionError::duplicate("count", 0).to_string(),
            "duplicate definition `count` at offset 0"
        );
        assert_eq!(
            ResolutionError::missing_loop_key(7).to_string(),
            "loop is missing a `key` attribute at offset 7"
        );
    }

    #[test]
    fn accessors_expose_fields() {
        let err = ResolutionError::unknown_name("x", 12);
        assert_eq!(err.kind(), &ResolutionErrorKind::UnknownName);
        assert_eq!(err.name(), "x");
        assert_eq!(err.offset(), 12);
    }

    #[test]
    fn usable_as_std_error() {
        let err: &dyn std::error::Error = &ResolutionError::unknown_name("x", 0);
        assert_eq!(err.to_string(), "unknown name `x` at offset 0");
    }
}
