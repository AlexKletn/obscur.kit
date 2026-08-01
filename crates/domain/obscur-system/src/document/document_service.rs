#![doc = include_str!("document_service.doc/_module.md")]

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::document::{DirectoryHandle, DocumentHandle, DocumentMetadata, DocumentPermission};

#[doc = include_str!("document_service.doc/DocumentError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentError {
    #[doc = include_str!("document_service.doc/NotFound.md")]
    NotFound,
    #[doc = include_str!("document_service.doc/PermissionDenied.md")]
    PermissionDenied,
    #[doc = include_str!("document_service.doc/Unsupported.md")]
    Unsupported(Box<str>),
    #[doc = include_str!("document_service.doc/Io.md")]
    Io(Box<str>),
}

impl std::fmt::Display for DocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => f.write_str("document not found"),
            Self::PermissionDenied => f.write_str("document permission denied"),
            Self::Unsupported(what) => write!(f, "unsupported document operation: {what}"),
            Self::Io(reason) => write!(f, "document i/o error: {reason}"),
        }
    }
}

impl std::error::Error for DocumentError {}

#[doc = include_str!("document_service.doc/DocumentService.md")]
pub trait DocumentService {
    #[doc = include_str!("document_service.doc/read_bytes.md")]
    fn read_bytes(&self, handle: &DocumentHandle) -> Result<Arc<[u8]>, DocumentError>;

    #[doc = include_str!("document_service.doc/write_bytes.md")]
    fn write_bytes(&mut self, handle: &DocumentHandle, bytes: &[u8]) -> Result<(), DocumentError>;

    #[doc = include_str!("document_service.doc/list.md")]
    fn list(&self, dir: &DirectoryHandle) -> Result<Vec<DocumentHandle>, DocumentError>;

    #[doc = include_str!("document_service.doc/metadata.md")]
    fn metadata(&self, handle: &DocumentHandle) -> Result<DocumentMetadata, DocumentError>;
}

#[doc = include_str!("document_service.doc/MemoryDocumentService.md")]
#[derive(Debug, Default, Clone)]
pub struct MemoryDocumentService {
    #[doc = include_str!("document_service.doc/files.md")]
    files: BTreeMap<Box<str>, Arc<[u8]>>,
    #[doc = include_str!("document_service.doc/dirs.md")]
    dirs: BTreeMap<Box<str>, Vec<Box<str>>>,
}

impl MemoryDocumentService {
    #[doc = include_str!("document_service.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("document_service.doc/insert_file.md")]
    pub fn insert_file(&mut self, token: impl Into<Box<str>>, bytes: impl Into<Arc<[u8]>>) {
        self.files.insert(token.into(), bytes.into());
    }

    #[doc = include_str!("document_service.doc/insert_dir.md")]
    pub fn insert_dir(&mut self, token: impl Into<Box<str>>, children: Vec<Box<str>>) {
        self.dirs.insert(token.into(), children);
    }
}

impl DocumentService for MemoryDocumentService {
    fn read_bytes(&self, handle: &DocumentHandle) -> Result<Arc<[u8]>, DocumentError> {
        self.files
            .get(handle.token())
            .cloned()
            .ok_or(DocumentError::NotFound)
    }

    fn write_bytes(&mut self, handle: &DocumentHandle, bytes: &[u8]) -> Result<(), DocumentError> {
        if !handle.can_write() {
            return Err(DocumentError::PermissionDenied);
        }
        self.files
            .insert(Box::from(handle.token()), Arc::from(bytes));
        Ok(())
    }

    fn list(&self, dir: &DirectoryHandle) -> Result<Vec<DocumentHandle>, DocumentError> {
        let children = self.dirs.get(dir.token()).ok_or(DocumentError::NotFound)?;
        let handles = children
            .iter()
            .map(|token| {
                DocumentHandle::new(
                    token.clone(),
                    DocumentMetadata::new(token.clone()),
                    DocumentPermission::read_only(),
                )
            })
            .collect();
        Ok(handles)
    }

    fn metadata(&self, handle: &DocumentHandle) -> Result<DocumentMetadata, DocumentError> {
        Ok(handle.metadata().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time proof the port is object-safe (spec §6.10 replaceable port).
    #[test]
    fn document_service_is_object_safe() {
        let _: Box<dyn DocumentService> = Box::new(MemoryDocumentService::new());
    }

    fn writable_handle(token: &str) -> DocumentHandle {
        DocumentHandle::new(
            token,
            DocumentMetadata::new(token),
            DocumentPermission::read_write(),
        )
    }

    #[test]
    fn write_then_read_round_trips() {
        let mut svc = MemoryDocumentService::new();
        let handle = writable_handle("doc-1");
        svc.write_bytes(&handle, b"hello").expect("writable");
        let bytes = svc.read_bytes(&handle).expect("present");
        assert_eq!(&*bytes, b"hello");
    }

    #[test]
    fn write_to_read_only_is_permission_denied() {
        let mut svc = MemoryDocumentService::new();
        let handle = DocumentHandle::new(
            "doc-2",
            DocumentMetadata::new("doc-2"),
            DocumentPermission::read_only(),
        );
        assert_eq!(
            svc.write_bytes(&handle, b"x"),
            Err(DocumentError::PermissionDenied)
        );
    }

    #[test]
    fn read_missing_is_not_found() {
        let svc = MemoryDocumentService::new();
        let handle = writable_handle("ghost");
        assert_eq!(svc.read_bytes(&handle), Err(DocumentError::NotFound));
    }

    #[test]
    fn list_returns_seeded_children() {
        let mut svc = MemoryDocumentService::new();
        svc.insert_file("a", Arc::<[u8]>::from(&b"aa"[..]));
        svc.insert_dir("dir", vec![Box::from("a"), Box::from("b")]);
        let dir = DirectoryHandle::new(
            "dir",
            DocumentMetadata::new("dir").directory(true),
            DocumentPermission::read_only(),
        );
        let listed = svc.list(&dir).expect("known dir");
        let names: Vec<&str> = listed.iter().map(DocumentHandle::token).collect();
        assert_eq!(names, ["a", "b"]);
        assert_eq!(listed[0].name(), "a");
        assert!(!listed[0].can_write());
    }

    #[test]
    fn list_unknown_dir_is_not_found() {
        let svc = MemoryDocumentService::new();
        let dir = DirectoryHandle::new(
            "nope",
            DocumentMetadata::new("nope").directory(true),
            DocumentPermission::read_only(),
        );
        assert_eq!(svc.list(&dir).unwrap_err(), DocumentError::NotFound);
    }

    #[test]
    fn metadata_returns_handle_metadata() {
        let svc = MemoryDocumentService::new();
        let handle = writable_handle("doc-3");
        assert_eq!(
            svc.metadata(&handle).unwrap(),
            DocumentMetadata::new("doc-3")
        );
    }

    #[test]
    fn document_error_displays_and_is_error() {
        assert_eq!(DocumentError::NotFound.to_string(), "document not found");
        assert_eq!(
            DocumentError::PermissionDenied.to_string(),
            "document permission denied"
        );
        assert_eq!(
            DocumentError::Unsupported("create".into()).to_string(),
            "unsupported document operation: create"
        );
        assert_eq!(
            DocumentError::Io("disk full".into()).to_string(),
            "document i/o error: disk full"
        );
        let _: &dyn std::error::Error = &DocumentError::NotFound;
    }
}
