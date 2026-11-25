//! PackageFormat (skeleton stub for the spec-defined entity; not yet implemented).

/// The archive formats provided by the streamed package reader.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PackageFormat {
    /// An uncompressed TAR archive.
    Tar,
    /// A gzip-compressed TAR archive.
    TarGz,
    /// A Zstandard-compressed TAR archive.
    TarZst,
    /// A ZIP archive.
    Zip,
}

impl PackageFormat {
    /// Detects the archive format from its file name.
    #[must_use]
    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy();
        [
            (".tar.gz", Self::TarGz),
            (".tar.zst", Self::TarZst),
            (".tar", Self::Tar),
            (".zip", Self::Zip),
        ]
        .into_iter()
        .find_map(|(suffix, format)| name.ends_with(suffix).then_some(format))
    }
}
