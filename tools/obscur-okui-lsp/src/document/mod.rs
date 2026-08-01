#![doc = include_str!("mod.doc/_module.md")]
mod document_store;
pub use document_store::DocumentStore;
mod text_document;
pub use text_document::{LineIndex, TextDocument};
