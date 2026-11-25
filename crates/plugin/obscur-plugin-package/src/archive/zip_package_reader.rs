//! Bounded ZIP reader for plugin packages.

use std::collections::BTreeSet;
use std::io::{self, Cursor, Read};

use zip::ZipArchive;
use zip::read::{ArchiveOffset, Config};
use zip::result::ZipError;

use super::byte_limit::ByteLimit;
use crate::package::{PackageEntry, PackageReader};
use crate::path::PackagePath;
use crate::validation::{PackageError, PackageLimits};

/// Reads regular files from a single-volume ZIP with UTF-8 entry names.
/// Supports stored and Deflate entries. Rejects ZIP64, encrypted entries,
/// and ZIP footer signatures preceding the archive footer.
#[derive(Debug)]
pub struct ZipPackageReader<R> {
    reader: R,
}

impl<R: Read> ZipPackageReader<R> {
    /// Wraps a ZIP input stream for bounded, in-memory package validation.
    #[must_use]
    pub const fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R: Read> PackageReader for ZipPackageReader<R> {
    fn read_entries(self, limits: PackageLimits) -> Result<Vec<PackageEntry>, PackageError> {
        let mut bytes = Vec::new();
        ByteLimit::new(self.reader, limits.archive_bytes).read_to_end(&mut bytes)?;
        let (footer, directory_offset, count) = check_directory(&bytes, limits.entries)?;
        // The comment is irrelevant to packages. Remove it before the ZIP
        // library scans for the footer so it cannot select a record within it.
        bytes.truncate(footer + 22);
        bytes[footer + 20..].fill(0);
        let config = Config {
            archive_offset: ArchiveOffset::Known(0),
        };
        let mut archive = ZipArchive::with_config(config, Cursor::new(bytes)).map_err(zip_error)?;
        if archive.central_directory_start() != directory_offset as u64 {
            return Err(invalid_zip("unexpected ZIP directory offset"));
        }
        // The library indexes entries by raw name and collapses exact duplicates.
        if archive.len() != count {
            return Err(PackageError::DuplicatePath("ZIP central directory".into()));
        }

        let mut files = Vec::new();
        let mut seen = BTreeSet::new();
        let mut remaining = limits.expanded_bytes;
        for index in 0..count {
            let entry = archive.by_index_raw(index).map_err(zip_error)?;
            if entry
                .data_start()
                .and_then(|start| start.checked_add(entry.compressed_size()))
                .is_none_or(|end| end > directory_offset as u64)
            {
                return Err(invalid_zip("ZIP entry extends into the central directory"));
            }
            let raw = entry.name_raw();
            let name = std::str::from_utf8(raw)
                .map_err(|_| PackageError::InvalidPath(String::from_utf8_lossy(raw).into()))?;
            let directory = entry.is_dir();
            let size = entry.size();
            let mode = entry.unix_mode().unwrap_or_default() & 0o170000;
            let expected_mode = if directory { 0o040000 } else { 0o100000 };
            if entry.encrypted() || (mode != 0 && mode != expected_mode) || (directory && size != 0)
            {
                return Err(PackageError::UnsupportedEntry(name.into()));
            }
            if directory && matches!(name, "." | "./") {
                continue;
            }
            let path =
                PackagePath::parse(name).ok_or_else(|| PackageError::InvalidPath(name.into()))?;
            if !seen.insert(path.clone()) {
                return Err(PackageError::DuplicatePath(path.as_str().into()));
            }
            if directory {
                continue;
            }
            if size > limits.entry_bytes {
                return Err(PackageError::Limit("entry size"));
            }
            if path.as_str() == "manifest.json"
                && usize::try_from(size).map_or(true, |size| size > limits.manifest_bytes)
            {
                return Err(PackageError::Limit("manifest size"));
            }
            remaining = remaining
                .checked_sub(size)
                .ok_or(PackageError::Limit("expanded size"))?;
            drop(entry);

            let mut contents = Vec::new();
            archive
                .by_index(index)
                .map_err(zip_error)?
                .take(size.saturating_add(1))
                .read_to_end(&mut contents)?;
            if u64::try_from(contents.len()) != Ok(size) {
                return Err(invalid_zip(
                    "ZIP entry size differs from its central directory",
                ));
            }
            files.push(PackageEntry::file(path, contents));
        }
        Ok(files)
    }
}

fn check_directory(
    bytes: &[u8],
    max_entries: usize,
) -> Result<(usize, usize, usize), PackageError> {
    // The ZIP library can retry earlier footer signatures after a parsing
    // error. Use the first signature and require its comment to end at EOF.
    let footer = bytes
        .windows(4)
        .position(|signature| signature == b"PK\x05\x06")
        .ok_or_else(|| invalid_zip("missing ZIP footer"))?;
    let record = bytes
        .get(footer..)
        .filter(|record| record.len() >= 22)
        .ok_or_else(|| invalid_zip("truncated ZIP footer"))?;
    let short = |offset| u16::from_le_bytes([record[offset], record[offset + 1]]);
    let word = |offset| u32::from(short(offset)) | (u32::from(short(offset + 2)) << 16);
    let count = usize::from(short(10));
    let directory = word(16) as usize;
    let size = word(12) as usize;
    if record.len() != 22 + usize::from(short(20))
        || short(4) != 0
        || short(6) != 0
        || short(8) != short(10)
        || short(10) == u16::MAX
        || word(12) == u32::MAX
        || word(16) == u32::MAX
        || directory.checked_add(size) != Some(footer)
        || count * 46 > size
    {
        return Err(invalid_zip("invalid or unsupported ZIP directory"));
    }
    if count > max_entries {
        return Err(PackageError::Limit("entry count"));
    }
    let mut records = &bytes[directory..footer];
    for _ in 0..count {
        let header = records
            .get(..46)
            .filter(|header| header.starts_with(b"PK\x01\x02"))
            .ok_or_else(|| invalid_zip("invalid ZIP directory entry"))?;
        if header[34..36] != [0, 0] {
            return Err(invalid_zip("ZIP entry references an external disk"));
        }
        let field_len =
            |offset| usize::from(u16::from_le_bytes([header[offset], header[offset + 1]]));
        let len = 46 + field_len(28) + field_len(30) + field_len(32);
        records = records
            .get(len..)
            .ok_or_else(|| invalid_zip("truncated ZIP directory entry"))?;
    }
    if !records.is_empty() {
        return Err(invalid_zip(
            "ZIP directory length differs from its entry count",
        ));
    }
    Ok((footer, directory, count))
}

fn invalid_zip(message: &'static str) -> PackageError {
    io::Error::new(io::ErrorKind::InvalidData, message).into()
}

fn zip_error(error: ZipError) -> PackageError {
    io::Error::new(io::ErrorKind::InvalidData, error).into()
}
