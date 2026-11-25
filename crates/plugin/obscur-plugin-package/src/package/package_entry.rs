//! PackageEntry (skeleton stub for the spec-defined entity; not yet implemented).

use crate::path::PackagePath;

/// One regular file from a package reader.
#[derive(Debug)]
pub struct PackageEntry {
    path: PackagePath,
    bytes: Vec<u8>,
}

impl PackageEntry {
    /// Constructs a decoded file from a canonical path and its contents.
    #[must_use]
    pub const fn file(path: PackagePath, bytes: Vec<u8>) -> Self {
        Self { path, bytes }
    }

    /// The normalized logical path of this file.
    #[must_use]
    pub const fn path(&self) -> &PackagePath {
        &self.path
    }

    /// The decoded payload bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
