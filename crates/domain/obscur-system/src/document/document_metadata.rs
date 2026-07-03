//! Descriptive metadata for a document or directory (spec §6.10 metadata).

/// Descriptive metadata a host reports for a document or directory
/// (spec §6.10 metadata).
///
/// A file picker hands back a handle plus descriptive metadata: a display
/// `name` (spec §6.10 — a name for presentation, *not* a filesystem path), an
/// optional MIME type, byte size, and last-modified time, and whether the handle
/// refers to a directory. Every field beyond the name is optional because a
/// portal, cloud document, or virtual filesystem may not expose it — such
/// details are reported as `None` rather than fabricated.
///
/// This is pure data: `modified_unix_secs` is a plain Unix timestamp supplied by
/// the host, never read from a clock here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentMetadata {
    name: Box<str>,
    mime_type: Option<Box<str>>,
    byte_size: Option<u64>,
    modified_unix_secs: Option<i64>,
    is_directory: bool,
}

impl DocumentMetadata {
    /// Creates metadata with only a display name; all other fields are unset
    /// (spec §6.10).
    ///
    /// The name is for presentation, not a filesystem path.
    #[must_use]
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            mime_type: None,
            byte_size: None,
            modified_unix_secs: None,
            is_directory: false,
        }
    }

    /// Returns this metadata with its MIME type set (spec §6.10).
    #[must_use]
    pub fn with_mime_type(mut self, mime_type: impl Into<Box<str>>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    /// Returns this metadata with its byte size set (spec §6.10).
    #[must_use]
    pub const fn with_byte_size(mut self, byte_size: u64) -> Self {
        self.byte_size = Some(byte_size);
        self
    }

    /// Returns this metadata with its last-modified Unix timestamp set
    /// (spec §6.10).
    ///
    /// The value is a host-supplied Unix timestamp in seconds, not read from a
    /// clock here.
    #[must_use]
    pub const fn with_modified(mut self, modified_unix_secs: i64) -> Self {
        self.modified_unix_secs = Some(modified_unix_secs);
        self
    }

    /// Returns this metadata with its directory flag set (spec §6.10).
    #[must_use]
    pub const fn directory(mut self, is_directory: bool) -> Self {
        self.is_directory = is_directory;
        self
    }

    /// Returns the display name (spec §6.10 — not a filesystem path).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the MIME type, if known (spec §6.10).
    #[must_use]
    pub fn mime_type(&self) -> Option<&str> {
        self.mime_type.as_deref()
    }

    /// Returns the byte size, if known (spec §6.10).
    #[must_use]
    pub const fn byte_size(&self) -> Option<u64> {
        self.byte_size
    }

    /// Returns the last-modified Unix timestamp in seconds, if known
    /// (spec §6.10).
    #[must_use]
    pub const fn modified_unix_secs(&self) -> Option<i64> {
        self.modified_unix_secs
    }

    /// Returns whether the handle refers to a directory (spec §6.10).
    #[must_use]
    pub const fn is_directory(&self) -> bool {
        self.is_directory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_only_a_name() {
        let meta = DocumentMetadata::new("Report.pdf");
        assert_eq!(meta.name(), "Report.pdf");
        assert_eq!(meta.mime_type(), None);
        assert_eq!(meta.byte_size(), None);
        assert_eq!(meta.modified_unix_secs(), None);
        assert!(!meta.is_directory());
    }

    #[test]
    fn builder_sets_every_field() {
        let meta = DocumentMetadata::new("Report.pdf")
            .with_mime_type("application/pdf")
            .with_byte_size(2048)
            .with_modified(1_700_000_000)
            .directory(false);
        assert_eq!(meta.mime_type(), Some("application/pdf"));
        assert_eq!(meta.byte_size(), Some(2048));
        assert_eq!(meta.modified_unix_secs(), Some(1_700_000_000));
        assert!(!meta.is_directory());
    }

    #[test]
    fn directory_flag_is_honored() {
        let meta = DocumentMetadata::new("Photos").directory(true);
        assert!(meta.is_directory());
    }
}
