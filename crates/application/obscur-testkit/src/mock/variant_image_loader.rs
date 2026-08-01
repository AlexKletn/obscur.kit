#![doc = include_str!("variant_image_loader.doc/_module.md")]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use obscur_resources::port::{LoadedBytes, ResourceLoader};
use obscur_resources::source::ResourceSource;
use obscur_resources::state::{ResourceError, ResourceMetadata};

#[doc = include_str!("variant_image_loader.doc/VariantImageLoader.md")]
#[derive(Debug, Clone)]
pub struct VariantImageLoader {
    #[doc = include_str!("variant_image_loader.doc/files.md")]
    files: HashMap<Box<str>, u8>,
    #[doc = include_str!("variant_image_loader.doc/asked.md")]
    asked: Arc<Mutex<Vec<Box<str>>>>,
}

impl VariantImageLoader {
    #[doc = include_str!("variant_image_loader.doc/BOTH_VARIANTS.md")]
    pub const BOTH_VARIANTS: &'static [(&'static str, u8)] = &[("x", 2), ("x@2x", 4)];

    #[doc = include_str!("variant_image_loader.doc/new.md")]
    #[must_use]
    pub fn new(files: &[(&str, u8)]) -> Self {
        Self {
            files: files.iter().map(|(p, n)| ((*p).into(), *n)).collect(),
            asked: Arc::new(Mutex::new(Vec::new())),
        }
    }

    #[doc = include_str!("variant_image_loader.doc/asked_fn.md")]
    #[must_use]
    pub fn asked(&self) -> Vec<Box<str>> {
        self.asked.lock().expect("uncontended").clone()
    }
}

impl ResourceLoader for VariantImageLoader {
    fn can_load(&self, source: &ResourceSource) -> bool {
        source.is_file()
    }

    fn load(&self, source: &ResourceSource) -> Result<LoadedBytes, ResourceError> {
        let ResourceSource::File(path) = source else {
            return Err(ResourceError::Unsupported("not a file".into()));
        };
        self.asked.lock().expect("uncontended").push(path.clone());
        let dimension = self
            .files
            .get(path)
            .copied()
            .ok_or(ResourceError::NotFound)?;
        let bytes: Arc<[u8]> = Arc::from(vec![dimension]);
        Ok(LoadedBytes::new(
            bytes,
            ResourceMetadata::new().with_byte_size(1),
        ))
    }
}
