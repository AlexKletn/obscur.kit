#![doc = include_str!("../../docs/api/port/resource_loader.doc/_module.md")]

use crate::source::ResourceSource;
use crate::state::ResourceError;

use super::LoadedBytes;

#[doc = include_str!("../../docs/api/port/resource_loader.doc/ResourceLoader.md")]
pub trait ResourceLoader {
    #[doc = include_str!("../../docs/api/port/resource_loader.doc/can_load.md")]
    #[must_use]
    fn can_load(&self, source: &ResourceSource) -> bool;

    #[doc = include_str!("../../docs/api/port/resource_loader.doc/load.md")]
    fn load(&self, source: &ResourceSource) -> Result<LoadedBytes, ResourceError>;
}

#[cfg(test)]
mod tests {
    use super::super::InlineLoader;
    use super::*;

    /// Compile-time proof the port is object-safe: adapters must be storable as
    /// `Box<dyn ResourceLoader>`.
    #[test]
    fn resource_loader_is_object_safe() {
        let loader: Box<dyn ResourceLoader> = Box::new(InlineLoader::new());
        assert!(!loader.can_load(&ResourceSource::file("/x")));
    }
}
