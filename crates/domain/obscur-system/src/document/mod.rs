//! Document module.
mod directory_handle;
pub use directory_handle::DirectoryHandle;
mod document_handle;
pub use document_handle::DocumentHandle;
mod document_metadata;
pub use document_metadata::DocumentMetadata;
mod document_permission;
pub use document_permission::DocumentPermission;
mod access_mode;
pub use access_mode::AccessMode;
mod document_error;
pub use document_error::DocumentError;
mod document_service;
pub use document_service::DocumentService;
mod memory_document_service;
pub use memory_document_service::MemoryDocumentService;
