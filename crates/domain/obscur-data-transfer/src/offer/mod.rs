//! Offer module.
mod data_offer;
pub use data_offer::DataOffer;
mod data_representation;
pub use data_representation::DataRepresentation;
mod lazy_data_provider;
pub use lazy_data_provider::LazyDataProvider;
mod static_data_provider;
pub use static_data_provider::StaticDataProvider;
mod mime_type;
pub use mime_type::MimeType;
