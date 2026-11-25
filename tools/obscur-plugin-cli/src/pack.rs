//! Builds and validates plugin archives from a local build directory.

use std::{
    error::Error,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use obscur_plugin_package::{
    archive::{TarPackageReader, ZipPackageReader},
    package::PackageFormat,
    path::PackagePath,
    validation::{PackageLimits, PackageValidator},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

pub(crate) fn build(
    source: &Path,
    destination: &Path,
    validator: &PackageValidator,
) -> Result<(), Box<dyn Error>> {
    let zip = if destination
        .extension()
        .is_some_and(|extension| extension == "zip")
    {
        true
    } else if destination
        .extension()
        .is_some_and(|extension| extension == "tar")
    {
        false
    } else {
        return Err("pack output must end in .tar or .zip".into());
    };
    let limits = PackageLimits::default();
    let source = source.canonicalize()?;
    let mut manifest: Value = serde_json::from_slice(&read_entry(
        &source,
        "manifest.json",
        limits.manifest_bytes as u64,
    )?)?;
    let rows = manifest
        .get_mut("files")
        .and_then(Value::as_array_mut)
        .ok_or("manifest files must be an array")?;
    if rows.len() >= limits.entries {
        return Err("too many package files".into());
    }
    let mut entries = Vec::with_capacity(rows.len() + 1);
    let mut remaining = limits.expanded_bytes;
    for row in rows {
        let path = row
            .get("path")
            .and_then(Value::as_str)
            .and_then(PackagePath::parse)
            .filter(|path| path.as_str() != "manifest.json")
            .ok_or("invalid package file path")?;
        let bytes = read_entry(&source, path.as_str(), limits.entry_bytes.min(remaining))?;
        remaining -= bytes.len() as u64;
        row["sha256"] = json!(format!("{:x}", Sha256::digest(&bytes)));
        entries.push((path.as_str().to_owned(), bytes));
    }
    entries.push(("manifest.json".into(), serde_json::to_vec(&manifest)?));

    let directory = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut archive = NamedTempFile::new_in(directory)?;
    if zip {
        let mut writer = zip::ZipWriter::new(archive.as_file_mut());
        for (path, bytes) in entries {
            writer.start_file(
                path,
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated),
            )?;
            writer.write_all(&bytes)?;
        }
        writer.finish()?;
        validator.load(ZipPackageReader::new(archive.reopen()?))?;
    } else {
        let mut writer = tar::Builder::new(archive.as_file_mut());
        for (path, bytes) in entries {
            let mut header = tar::Header::new_ustar();
            header.set_mode(0o644);
            header.set_size(bytes.len() as u64);
            header.set_entry_type(tar::EntryType::Regular);
            header.set_cksum();
            writer.append_data(&mut header, path, bytes.as_slice())?;
        }
        writer.into_inner()?;
        validator.load(TarPackageReader::new(archive.reopen()?, PackageFormat::Tar))?;
    }
    archive.persist_noclobber(destination)?;
    Ok(())
}

fn read_entry(source: &Path, name: &str, limit: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let path = fs::canonicalize(source.join(name))?;
    if !path.starts_with(source) || !fs::metadata(&path)?.is_file() {
        return Err(
            format!("package input {name:?} is outside the source or is not a file").into(),
        );
    }
    let mut bytes = Vec::new();
    File::open(path)?.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(format!("package input {name:?} exceeds its byte limit").into());
    }
    Ok(bytes)
}
