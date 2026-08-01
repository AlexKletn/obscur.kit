#![doc = include_str!("resolution_error.doc/_module.md")]

#[doc = include_str!("resolution_error.doc/ResolutionErrorKind.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionErrorKind {
    #[doc = include_str!("resolution_error.doc/UnknownName.md")]
    UnknownName,
    #[doc = include_str!("resolution_error.doc/DuplicateDefinition.md")]
    DuplicateDefinition,
    #[doc = include_str!("resolution_error.doc/MissingLoopKey.md")]
    MissingLoopKey,
}

#[doc = include_str!("resolution_error.doc/ResolutionError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionError {
    kind: ResolutionErrorKind,
    name: Box<str>,
    offset: u32,
}

impl ResolutionError {
    #[doc = include_str!("resolution_error.doc/new.md")]
    #[must_use]
    pub fn new(kind: ResolutionErrorKind, name: impl Into<Box<str>>, offset: u32) -> Self {
        Self {
            kind,
            name: name.into(),
            offset,
        }
    }

    #[doc = include_str!("resolution_error.doc/unknown_name.md")]
    #[must_use]
    pub fn unknown_name(name: impl Into<Box<str>>, offset: u32) -> Self {
        Self::new(ResolutionErrorKind::UnknownName, name, offset)
    }

    #[doc = include_str!("resolution_error.doc/duplicate.md")]
    #[must_use]
    pub fn duplicate(name: impl Into<Box<str>>, offset: u32) -> Self {
        Self::new(ResolutionErrorKind::DuplicateDefinition, name, offset)
    }

    #[doc = include_str!("resolution_error.doc/missing_loop_key.md")]
    #[must_use]
    pub fn missing_loop_key(offset: u32) -> Self {
        Self::new(ResolutionErrorKind::MissingLoopKey, "key", offset)
    }

    #[doc = include_str!("resolution_error.doc/kind.md")]
    #[must_use]
    pub const fn kind(&self) -> &ResolutionErrorKind {
        &self.kind
    }

    #[doc = include_str!("resolution_error.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[doc = include_str!("resolution_error.doc/offset.md")]
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
