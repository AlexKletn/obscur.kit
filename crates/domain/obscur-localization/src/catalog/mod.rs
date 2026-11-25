//! Catalog module.
mod message_provider;
pub use message_provider::MessageProvider;
mod empty_message_provider;
pub use empty_message_provider::EmptyMessageProvider;
mod message_catalog;
pub use message_catalog::MessageCatalog;
mod localizer;
pub use localizer::Localizer;
