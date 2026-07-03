//! A filesystem-backed [`ResourceLoader`] adapter (spec §5.15 sources, §2.7
//! adapters — "Filesystem loader").
//!
//! Spec §5.15 lists `file` among a resource's sources, and states the kit's core
//! performs no filesystem access itself — an interface adapter does. This module
//! is that adapter: [`FsResourceLoader`] implements the pure
//! [`ResourceLoader`] port using only `std`, reading the encoded bytes behind a
//! [`ResourceSource::File`] from disk. An optional sandbox root confines resolved
//! paths to a directory subtree (spec §2.7 adapters keep host reach explicit).

use std::path::PathBuf;
use std::sync::Arc;

use obscur_resources::port::{LoadedBytes, ResourceLoader};
use obscur_resources::source::ResourceSource;
use obscur_resources::state::{ResourceError, ResourceMetadata};

/// A filesystem-backed [`ResourceLoader`] that reads bytes for
/// [`ResourceSource::File`] paths from disk (spec §5.15 file, §2.7 filesystem
/// loader).
///
/// This adapter handles only filesystem paths: the bytes-in-hand kinds
/// ([`ResourceSource::Embedded`], [`ResourceSource::Memory`]) are the
/// `InlineLoader`'s job, and [`ResourceSource::Url`] / [`ResourceSource::Document`]
/// require a host loader. When constructed with a sandbox root via
/// [`FsResourceLoader::with_root`], `File` paths resolve relative to that root and
/// may not escape it through `..` traversal; without a root ([`FsResourceLoader::new`])
/// the path is used as given (absolute or relative to the process working
/// directory).
#[derive(Debug, Clone)]
pub struct FsResourceLoader {
    /// The optional sandbox root. When `Some`, `File` paths resolve relative to
    /// it and must stay within its canonicalized subtree.
    root: Option<PathBuf>,
}

impl FsResourceLoader {
    /// Creates a loader with no sandbox root: `File` paths are used as-is
    /// (absolute, or relative to the process working directory).
    #[must_use]
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Creates a loader confined to `root`: every `File` path resolves relative
    /// to `root` and is rejected if it escapes the canonicalized root subtree
    /// (spec §5.15 file, §2.7).
    #[must_use]
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Self {
            root: Some(root.into()),
        }
    }
}

impl Default for FsResourceLoader {
    /// The default loader is [`FsResourceLoader::new`] (no sandbox root).
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceLoader for FsResourceLoader {
    fn can_load(&self, source: &ResourceSource) -> bool {
        matches!(source, ResourceSource::File(_))
    }

    fn load(&self, source: &ResourceSource) -> Result<LoadedBytes, ResourceError> {
        let ResourceSource::File(path) = source else {
            return Err(ResourceError::Unsupported(
                "fs loader only handles File sources".into(),
            ));
        };

        let resolved = match &self.root {
            Some(base) => {
                let joined = base.join(path.as_ref());
                // Sandbox: the resolved path, once canonicalized, must remain
                // within the canonicalized root; a `..` escape is refused. A
                // canonicalize failure because the file is absent maps to
                // NotFound (there is nothing to escape with).
                let canonical_root = std::fs::canonicalize(base).map_err(|io| map_io_error(&io))?;
                let canonical_target =
                    std::fs::canonicalize(&joined).map_err(|io| map_io_error(&io))?;
                if !canonical_target.starts_with(&canonical_root) {
                    return Err(ResourceError::SourceUnavailable(
                        "path escapes loader root".into(),
                    ));
                }
                canonical_target
            }
            None => PathBuf::from(path.as_ref()),
        };

        let bytes = std::fs::read(&resolved).map_err(|io| map_io_error(&io))?;

        let mut metadata =
            ResourceMetadata::new().with_byte_size(u64::try_from(bytes.len()).unwrap_or(u64::MAX));
        if let Some(mime) = resolved
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(mime_for_extension)
        {
            metadata = metadata.with_mime_type(mime);
        }

        Ok(LoadedBytes::new(Arc::from(bytes), metadata))
    }
}

/// Maps a filesystem [`std::io::Error`] onto a [`ResourceError`] (spec §5.15
/// failed state): a missing file is [`ResourceError::NotFound`], a permission
/// failure is a [`ResourceError::SourceUnavailable`] with a fixed reason, and any
/// other I/O failure is [`ResourceError::SourceUnavailable`] carrying the I/O
/// error's own message.
fn map_io_error(err: &std::io::Error) -> ResourceError {
    match err.kind() {
        std::io::ErrorKind::NotFound => ResourceError::NotFound,
        std::io::ErrorKind::PermissionDenied => {
            ResourceError::SourceUnavailable("permission denied".into())
        }
        _ => ResourceError::SourceUnavailable(err.to_string().into()),
    }
}

/// Returns a MIME type for a file `extension`, or `None` when it is not in this
/// minimal map.
///
/// This is a deliberately small, best-effort table covering the common UI asset
/// extensions this adapter is expected to serve (images, vector art, stylesheets,
/// JSON, plain text, and web fonts). It is not a comprehensive MIME registry;
/// callers that need authoritative typing should sniff the bytes. The lookup is
/// case-insensitive.
fn mime_for_extension(extension: &str) -> Option<&'static str> {
    let lower = extension.to_ascii_lowercase();
    let mime = match lower.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "css" => "text/css",
        "json" => "application/json",
        "txt" => "text/plain",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        _ => return None,
    };
    Some(mime)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::atomic::{AtomicU32, Ordering};

    use obscur_resources::source::MemorySource;

    /// Process-unique counter so concurrent tests never collide on a temp name.
    static COUNTER: AtomicU32 = AtomicU32::new(0);

    /// Creates a fresh, empty temp directory unique to this process and test.
    fn unique_temp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "obscur-resource-fs-{}-{n}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    /// Removes a temp directory tree, ignoring absence.
    fn cleanup(dir: &Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn loads_file_with_bytes_size_and_mime() {
        let dir = unique_temp_dir();
        let file = dir.join("asset.png");
        let payload = b"\x89PNG\r\n\x1a\n payload bytes";
        std::fs::write(&file, payload).expect("write temp file");

        let loader = FsResourceLoader::new();
        let source = ResourceSource::file(file.to_string_lossy().into_owned());
        assert!(loader.can_load(&source));

        let loaded = loader.load(&source).expect("file loads");
        assert_eq!(&**loaded.bytes(), payload);
        assert_eq!(loaded.metadata().byte_size(), Some(payload.len() as u64));
        assert_eq!(loaded.metadata().mime_type(), Some("image/png"));

        cleanup(&dir);
    }

    #[test]
    fn missing_path_is_not_found() {
        let dir = unique_temp_dir();
        let missing = dir.join("does-not-exist.bin");

        let loader = FsResourceLoader::new();
        let source = ResourceSource::file(missing.to_string_lossy().into_owned());
        assert_eq!(loader.load(&source), Err(ResourceError::NotFound));

        cleanup(&dir);
    }

    #[test]
    fn can_load_true_for_file_false_for_memory() {
        let loader = FsResourceLoader::new();
        assert!(loader.can_load(&ResourceSource::file("/some/path.png")));
        assert!(!loader.can_load(&ResourceSource::memory(MemorySource::new(vec![1u8, 2]))));
    }

    #[test]
    fn non_file_source_is_unsupported() {
        let loader = FsResourceLoader::new();
        let source = ResourceSource::memory(MemorySource::new(vec![1u8, 2, 3]));
        assert!(matches!(
            loader.load(&source),
            Err(ResourceError::Unsupported(_))
        ));
    }

    #[test]
    fn path_escaping_root_is_source_unavailable() {
        let base = unique_temp_dir();
        let root = base.join("root");
        std::fs::create_dir_all(&root).expect("create root");

        // A real file that lives OUTSIDE the sandbox root, reachable via `..`.
        let outside = base.join("secret.txt");
        std::fs::write(&outside, b"secret").expect("write outside file");

        let loader = FsResourceLoader::with_root(&root);
        let source = ResourceSource::file("../secret.txt");
        assert_eq!(
            loader.load(&source),
            Err(ResourceError::SourceUnavailable(
                "path escapes loader root".into()
            ))
        );

        cleanup(&base);
    }

    #[test]
    fn with_root_loads_contained_file() {
        let root = unique_temp_dir();
        std::fs::write(root.join("inside.txt"), b"hello root").expect("write inside file");

        let loader = FsResourceLoader::with_root(&root);
        let loaded = loader
            .load(&ResourceSource::file("inside.txt"))
            .expect("contained file loads");
        assert_eq!(&**loaded.bytes(), b"hello root");
        assert_eq!(loaded.metadata().mime_type(), Some("text/plain"));

        cleanup(&root);
    }
}
