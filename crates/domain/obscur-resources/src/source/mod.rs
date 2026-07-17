//! Source module.
mod document_source;
pub use document_source::DocumentSource;
mod dpi_variant;
pub use dpi_variant::{dpi_variants, requested_density};
mod embedded_source;
pub use embedded_source::EmbeddedSource;
mod generated_source;
pub use generated_source::GeneratedSource;
mod memory_source;
pub use memory_source::MemorySource;
mod resource_source;
pub use resource_source::ResourceSource;
