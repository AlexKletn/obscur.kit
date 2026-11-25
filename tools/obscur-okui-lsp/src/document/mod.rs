//! Document model: the open-document [`DocumentStore`], each [`TextDocument`]
//! and its [`LineIndex`] for offset ↔ position mapping.
mod document_store;
pub use document_store::DocumentStore;
mod text_document;
pub use text_document::TextDocument;
mod line_index;
pub use line_index::LineIndex;
