#![doc = include_str!("fs_resource_loader.doc/_module.md")]

use std::path::PathBuf;
use std::sync::Arc;

use obscur_resources::port::{LoadedBytes, ResourceLoader};
use obscur_resources::source::ResourceSource;
use obscur_resources::state::{ResourceError, ResourceMetadata};

#[doc = include_str!("fs_resource_loader.doc/FsResourceLoader.md")]
#[derive(Debug, Clone)]
pub struct FsResourceLoader {
    #[doc = include_str!("fs_resource_loader.doc/root.md")]
    root: Option<PathBuf>,
}

impl FsResourceLoader {
    #[doc = include_str!("fs_resource_loader.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self { root: None }
    }

    #[doc = include_str!("fs_resource_loader.doc/with_root.md")]
    #[must_use]
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Self {
            root: Some(root.into()),
        }
    }
}

impl Default for FsResourceLoader {
    #[doc = include_str!("fs_resource_loader.doc/default.md")]
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

#[doc = include_str!("fs_resource_loader.doc/map_io_error.md")]
fn map_io_error(err: &std::io::Error) -> ResourceError {
    match err.kind() {
        std::io::ErrorKind::NotFound => ResourceError::NotFound,
        std::io::ErrorKind::PermissionDenied => {
            ResourceError::SourceUnavailable("permission denied".into())
        }
        _ => ResourceError::SourceUnavailable(err.to_string().into()),
    }
}

#[doc = include_str!("fs_resource_loader.doc/mime_for_extension.md")]
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
        let dir =
            std::env::temp_dir().join(format!("obscur-resource-fs-{}-{n}", std::process::id()));
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
