//! Mock providers a fixture installs in place of a real host's.
mod canned_image_decoder;
mod canned_image_loader;
mod queued_executor;
mod variant_image_decoder;
mod variant_image_loader;

pub use canned_image_decoder::CannedImageDecoder;
pub use canned_image_loader::CannedImageLoader;
pub use queued_executor::QueuedExecutor;
pub use variant_image_decoder::VariantImageDecoder;
pub use variant_image_loader::VariantImageLoader;
