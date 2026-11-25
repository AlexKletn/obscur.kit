use std::error::Error;
use std::fmt;
use std::io;

use super::ManifestError;

/// A typed failure during package decoding or policy validation.
#[derive(Debug)]
pub enum PackageError {
    /// The archive cannot be read within its byte budgets.
    Io(io::Error),
    /// A path cannot be safely normalized.
    InvalidPath(String),
    /// The archive contains a link, special entry, or unsupported record.
    UnsupportedEntry(String),
    /// A host-controlled resource limit was exceeded.
    Limit(&'static str),
    /// Two archive entries or hash declarations normalize to the same path.
    DuplicatePath(String),
    /// The manifest syntax or shape is invalid.
    Manifest(ManifestError),
    /// A manifest reference has no corresponding payload file.
    MissingFile(String),
    /// A payload file is not covered by a manifest hash.
    UnlistedFile(String),
    /// A payload differs from its declared SHA-256 hash.
    HashMismatch(String),
    /// A host version or target does not satisfy the declared requirements.
    Incompatible(&'static str),
    /// The manifest requests a capability the host did not authorize.
    DeniedCapability(String),
    /// A provider could not satisfy the declared dependency.
    Dependency(String),
}

impl fmt::Display for PackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::InvalidPath(path) => write!(f, "invalid package path {path:?}"),
            Self::UnsupportedEntry(path) => write!(f, "unsupported archive entry {path:?}"),
            Self::Limit(limit) => write!(f, "package {limit} limit exceeded"),
            Self::DuplicatePath(path) => write!(f, "duplicate package path {path:?}"),
            Self::Manifest(message) => write!(f, "invalid package manifest: {message}"),
            Self::MissingFile(path) => write!(f, "required package file {path:?} is missing"),
            Self::UnlistedFile(path) => write!(f, "package file {path:?} has no declared hash"),
            Self::HashMismatch(path) => write!(f, "SHA-256 mismatch for package file {path:?}"),
            Self::Incompatible(interface) => {
                write!(f, "package is incompatible with host {interface}")
            }
            Self::DeniedCapability(name) => write!(f, "package capability {name:?} is not allowed"),
            Self::Dependency(message) => write!(f, "package dependency: {message}"),
        }
    }
}

impl Error for PackageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Manifest(error) => error.source(),
            _ => None,
        }
    }
}

impl From<io::Error> for PackageError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for PackageError {
    fn from(error: serde_json::Error) -> Self {
        Self::Manifest(ManifestError::Decode(error))
    }
}
