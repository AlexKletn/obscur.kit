#![doc = include_str!("document_metadata.doc/_module.md")]

#[doc = include_str!("document_metadata.doc/DocumentMetadata.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentMetadata {
    name: Box<str>,
    mime_type: Option<Box<str>>,
    byte_size: Option<u64>,
    modified_unix_secs: Option<i64>,
    is_directory: bool,
}

impl DocumentMetadata {
    #[doc = include_str!("document_metadata.doc/new.md")]
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

    #[doc = include_str!("document_metadata.doc/with_mime_type.md")]
    #[must_use]
    pub fn with_mime_type(mut self, mime_type: impl Into<Box<str>>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    #[doc = include_str!("document_metadata.doc/with_byte_size.md")]
    #[must_use]
    pub const fn with_byte_size(mut self, byte_size: u64) -> Self {
        self.byte_size = Some(byte_size);
        self
    }

    #[doc = include_str!("document_metadata.doc/with_modified.md")]
    #[must_use]
    pub const fn with_modified(mut self, modified_unix_secs: i64) -> Self {
        self.modified_unix_secs = Some(modified_unix_secs);
        self
    }

    #[doc = include_str!("document_metadata.doc/directory.md")]
    #[must_use]
    pub const fn directory(mut self, is_directory: bool) -> Self {
        self.is_directory = is_directory;
        self
    }

    #[doc = include_str!("document_metadata.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[doc = include_str!("document_metadata.doc/mime_type.md")]
    #[must_use]
    pub fn mime_type(&self) -> Option<&str> {
        self.mime_type.as_deref()
    }

    #[doc = include_str!("document_metadata.doc/byte_size.md")]
    #[must_use]
    pub const fn byte_size(&self) -> Option<u64> {
        self.byte_size
    }

    #[doc = include_str!("document_metadata.doc/modified_unix_secs.md")]
    #[must_use]
    pub const fn modified_unix_secs(&self) -> Option<i64> {
        self.modified_unix_secs
    }

    #[doc = include_str!("document_metadata.doc/is_directory.md")]
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
