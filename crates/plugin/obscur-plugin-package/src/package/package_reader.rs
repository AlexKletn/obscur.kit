//! PackageReader (skeleton stub for the spec-defined entity; not yet implemented).

use super::PackageEntry;
use crate::validation::{PackageError, PackageLimits};

/// A trusted decoding adapter for a package archive format.
pub trait PackageReader {
    /// Streams the archive within the supplied byte and count limits, yielding regular files only.
    fn read_entries(self, limits: PackageLimits) -> Result<Vec<PackageEntry>, PackageError>;
}
