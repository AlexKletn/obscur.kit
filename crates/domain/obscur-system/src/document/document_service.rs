//! The replaceable port for document read/write/create/enumerate (spec §6.10).

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::document::{DirectoryHandle, DocumentHandle, DocumentMetadata, DocumentPermission};

/// Why a [`DocumentService`] operation failed (spec §6.10).
///
/// Each variant is a pure, backend-agnostic value; the informational variants
/// carry a human-readable reason. This type implements [`std::error::Error`] so
/// it composes with the standard error ecosystem while remaining I/O-free.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentError {
    /// No document or directory matched the handle's token.
    NotFound,
    /// The operation was not permitted — for example a write to a read-only
    /// handle (spec §6.10 permission token).
    PermissionDenied,
    /// The backend does not support this operation; the payload names what.
    Unsupported(Box<str>),
    /// An underlying I/O operation failed; the payload describes the failure.
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

/// The replaceable port for accessing documents behind opaque handles
/// (spec §6.10 read/write/create/enumerate).
///
/// The pure domain models document access as this object-safe trait: an adapter
/// implements it once per host (a real filesystem, a portal, a cloud provider, a
/// virtual filesystem) and the domain drives it as `Box<dyn DocumentService>`.
/// The trait is synchronous and returns [`Result`]; asynchronous and real I/O
/// concerns belong to the adapter, not here (spec §5.15).
///
/// Read-only operations take `&self`; mutating ones take `&mut self`. A write to
/// a handle that is not writable must fail with [`DocumentError::PermissionDenied`]
/// rather than silently succeeding.
pub trait DocumentService {
    /// Reads the full byte contents of the document behind `handle`
    /// (spec §6.10).
    ///
    /// # Errors
    ///
    /// Returns [`DocumentError::NotFound`] when the token resolves to nothing.
    fn read_bytes(&self, handle: &DocumentHandle) -> Result<Arc<[u8]>, DocumentError>;

    /// Writes `bytes` to the document behind `handle` (spec §6.10).
    ///
    /// # Errors
    ///
    /// Must return [`DocumentError::PermissionDenied`] when the handle is not
    /// writable ([`DocumentHandle::can_write`]).
    fn write_bytes(
        &mut self,
        handle: &DocumentHandle,
        bytes: &[u8],
    ) -> Result<(), DocumentError>;

    /// Enumerates the documents contained in `dir` (spec §6.10 enumerate).
    ///
    /// # Errors
    ///
    /// Returns [`DocumentError::NotFound`] when the directory token is unknown.
    fn list(&self, dir: &DirectoryHandle) -> Result<Vec<DocumentHandle>, DocumentError>;

    /// Returns the metadata for the document behind `handle` (spec §6.10).
    ///
    /// # Errors
    ///
    /// Returns [`DocumentError::NotFound`] when the token resolves to nothing.
    fn metadata(&self, handle: &DocumentHandle) -> Result<DocumentMetadata, DocumentError>;
}

/// A reference, in-memory [`DocumentService`] — a deterministic virtual
/// filesystem (spec §6.10).
///
/// `MemoryDocumentService` is the honest headless default: it stores document
/// bytes keyed by their opaque token and directory membership as token lists,
/// with no real I/O, clock, or randomness. It exists to exercise the port and to
/// give tests and headless runs a working document backend.
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

    fn write_bytes(
        &mut self,
        handle: &DocumentHandle,
        bytes: &[u8],
    ) -> Result<(), DocumentError> {
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
        assert_eq!(svc.metadata(&handle).unwrap(), DocumentMetadata::new("doc-3"));
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
