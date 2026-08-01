#![doc = include_str!("canned_image_loader.doc/_module.md")]

use std::sync::Arc;

use obscur_resources::port::{LoadedBytes, ResourceLoader};
use obscur_resources::source::ResourceSource;
use obscur_resources::state::{ResourceError, ResourceMetadata};

#[doc = include_str!("canned_image_loader.doc/CannedImageLoader.md")]
#[derive(Debug, Clone)]
pub struct CannedImageLoader {
    #[doc = include_str!("canned_image_loader.doc/fails.md")]
    fails: bool,
}

impl CannedImageLoader {
    #[doc = include_str!("canned_image_loader.doc/working.md")]
    #[must_use]
    pub fn working() -> Self {
        Self { fails: false }
    }

    #[doc = include_str!("canned_image_loader.doc/failing.md")]
    #[must_use]
    pub fn failing() -> Self {
        Self { fails: true }
    }
}

impl ResourceLoader for CannedImageLoader {
    fn can_load(&self, source: &ResourceSource) -> bool {
        source.is_file()
    }

    fn load(&self, _source: &ResourceSource) -> Result<LoadedBytes, ResourceError> {
        if self.fails {
            return Err(ResourceError::NotFound);
        }
        let bytes: Arc<[u8]> = Arc::from(vec![1u8, 2, 3, 4]);
        Ok(LoadedBytes::new(
            bytes,
            ResourceMetadata::new().with_byte_size(4),
        ))
    }
}
