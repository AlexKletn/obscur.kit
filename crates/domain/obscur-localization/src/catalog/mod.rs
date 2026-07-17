//! Catalog module.
mod message_provider;
pub use message_provider::{EmptyMessageProvider, MessageProvider};
mod message_catalog;
pub use message_catalog::MessageCatalog;
mod localizer;
pub use localizer::Localizer;
