use super::{
    DirectoryHandle, DocumentError, DocumentHandle, DocumentMetadata, DocumentPermission,
    DocumentService,
};
use std::collections::BTreeMap;
use std::sync::Arc;

#[doc = include_str!("../../docs/api/document/document_service.doc/MemoryDocumentService.md")]
#[derive(Debug, Default, Clone)]
pub struct MemoryDocumentService {
    /// Document bytes keyed by document token.
    files: BTreeMap<Box<str>, Arc<[u8]>>,
    /// Directory membership: directory token to its child document tokens.
    dirs: BTreeMap<Box<str>, Vec<Box<str>>>,
}

impl MemoryDocumentService {
    /// Creates an empty in-memory document service (spec §6.10).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seeds a document's bytes under `token` (test/adapter helper).
    pub fn insert_file(&mut self, token: impl Into<Box<str>>, bytes: impl Into<Arc<[u8]>>) {
        self.files.insert(token.into(), bytes.into());
    }

    /// Seeds a directory `token` with its child document tokens (test helper).
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
}
