//! The replaceable port for document read/write/create/enumerate (spec §6.10).

use super::{DirectoryHandle, DocumentError, DocumentHandle, DocumentMetadata};
use std::sync::Arc;

#[doc = include_str!("../../docs/api/document/document_service.doc/DocumentService.md")]
pub trait DocumentService {
    #[doc = include_str!("../../docs/api/document/document_service.doc/read_bytes.md")]
    fn read_bytes(&self, handle: &DocumentHandle) -> Result<Arc<[u8]>, DocumentError>;

    #[doc = include_str!("../../docs/api/document/document_service.doc/write_bytes.md")]
    fn write_bytes(&mut self, handle: &DocumentHandle, bytes: &[u8]) -> Result<(), DocumentError>;

    #[doc = include_str!("../../docs/api/document/document_service.doc/list.md")]
    fn list(&self, dir: &DirectoryHandle) -> Result<Vec<DocumentHandle>, DocumentError>;

    #[doc = include_str!("../../docs/api/document/document_service.doc/metadata.md")]
    fn metadata(&self, handle: &DocumentHandle) -> Result<DocumentMetadata, DocumentError>;
}

#[cfg(test)]
mod tests {
    use super::super::MemoryDocumentService;
    use super::*;

    #[test]
    fn document_service_is_object_safe() {
        let _: Box<dyn DocumentService> = Box::new(MemoryDocumentService::new());
    }
}
