//! PackageLimits (skeleton stub for the spec-defined entity; not yet implemented).

/// Host-controlled limits enforced while decoding a plugin package.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PackageLimits {
    /// Maximum input bytes read from the archive stream.
    pub archive_bytes: u64,
    /// Maximum decoded bytes. TAR includes framing and padding. ZIP counts decoded file contents.
    pub expanded_bytes: u64,
    /// Maximum declared regular-file size.
    pub entry_bytes: u64,
    /// Maximum number of archive entries, including directories.
    pub entries: usize,
    /// Maximum size of the manifest before JSON parsing.
    pub manifest_bytes: usize,
}

impl Default for PackageLimits {
    fn default() -> Self {
        Self {
            archive_bytes: 64 * 1024 * 1024,
            expanded_bytes: 128 * 1024 * 1024,
            entry_bytes: 32 * 1024 * 1024,
            entries: 4096,
            manifest_bytes: 256 * 1024,
        }
    }
}
