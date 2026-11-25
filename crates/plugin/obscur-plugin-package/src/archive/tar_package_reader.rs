//! Bounded TAR reader for plugin packages.

use std::collections::BTreeSet;
use std::io::{self, Read};

use flate2::read::MultiGzDecoder;

use super::byte_limit::ByteLimit;
use crate::package::{PackageEntry, PackageFormat, PackageReader};
use crate::path::PackagePath;
use crate::validation::{PackageError, PackageLimits};

/// A bounded reader for TAR and TAR compressed with gzip or Zstandard.
pub struct TarPackageReader<R> {
    reader: R,
    format: PackageFormat,
}

impl<R: Read> TarPackageReader<R> {
    /// Selects the decoding format without reading the archive.
    #[must_use]
    pub const fn new(reader: R, format: PackageFormat) -> Self {
        Self { reader, format }
    }
}

impl<R: Read> PackageReader for TarPackageReader<R> {
    fn read_entries(self, limits: PackageLimits) -> Result<Vec<PackageEntry>, PackageError> {
        let input = ByteLimit::new(self.reader, limits.archive_bytes);
        match self.format {
            PackageFormat::Tar => read_tar(input, limits),
            PackageFormat::TarGz => read_tar(MultiGzDecoder::new(input), limits),
            PackageFormat::TarZst => {
                let mut decoder = zstd::stream::read::Decoder::new(input)?;
                let window_log = (u64::BITS
                    - limits.expanded_bytes.saturating_sub(1).leading_zeros())
                .clamp(10, 27);
                decoder.window_log_max(window_log)?;
                read_tar(decoder, limits)
            }
            PackageFormat::Zip => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "use ZipPackageReader to read ZIP packages",
            )
            .into()),
        }
    }
}

fn read_tar(input: impl Read, limits: PackageLimits) -> Result<Vec<PackageEntry>, PackageError> {
    let mut archive = tar::Archive::new(ByteLimit::new(input, limits.expanded_bytes));
    let mut files = Vec::new();
    let mut seen = BTreeSet::new();
    // Raw mode avoids processing extended metadata records or sparse files
    // before their types and sizes have been checked.
    for (index, entry) in archive.entries()?.raw(true).enumerate() {
        if index >= limits.entries {
            return Err(PackageError::Limit("entry count"));
        }
        let mut entry = entry?;
        let path_bytes = entry.path_bytes();
        let path_text = std::str::from_utf8(&path_bytes)
            .map_err(|_| PackageError::InvalidPath(String::from_utf8_lossy(&path_bytes).into()))?;
        let kind = entry.header().entry_type();
        let size = entry.size();
        if kind.is_dir() && size == 0 && matches!(path_text, "." | "./") {
            continue;
        }
        let path = PackagePath::parse(path_text)
            .ok_or_else(|| PackageError::InvalidPath(path_text.into()))?;
        if !seen.insert(path.clone()) {
            return Err(PackageError::DuplicatePath(path.as_str().into()));
        }
        if kind.is_dir() && size == 0 {
            continue;
        }
        if !kind.is_file() {
            return Err(PackageError::UnsupportedEntry(path.as_str().into()));
        }
        if size > limits.entry_bytes || size > limits.expanded_bytes {
            return Err(PackageError::Limit("entry size"));
        }
        if path.as_str() == "manifest.json"
            && usize::try_from(size).map_or(true, |size| size > limits.manifest_bytes)
        {
            return Err(PackageError::Limit("manifest size"));
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        if u64::try_from(bytes.len()) != Ok(size) {
            return Err(
                io::Error::new(io::ErrorKind::UnexpectedEof, "truncated TAR payload").into(),
            );
        }
        files.push(PackageEntry::file(path, bytes));
    }

    // TAR iteration stops at the zero terminator. Drain through the same
    // limit to validate compression trailers and reject concealed trailing data.
    let mut input = archive.into_inner();
    let mut buffer = [0; 8192];
    loop {
        let n = input.read(&mut buffer)?;
        if n == 0 {
            return Ok(files);
        }
        if buffer[..n].iter().any(|&byte| byte != 0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected data after TAR terminator",
            )
            .into());
        }
    }
}
