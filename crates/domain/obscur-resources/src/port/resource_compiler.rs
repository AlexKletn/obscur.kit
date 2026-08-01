#![doc = include_str!("resource_compiler.doc/_module.md")]

use crate::handle::ResourceKind;
use crate::port::DecodedResource;
use crate::state::{ResourceError, ResourceMetadata};

#[doc = include_str!("resource_compiler.doc/CompiledResource.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledResource {
    byte_size: u64,
    metadata: ResourceMetadata,
}

impl CompiledResource {
    #[doc = include_str!("resource_compiler.doc/new.md")]
    #[must_use]
    pub fn new(byte_size: u64, metadata: ResourceMetadata) -> Self {
        Self {
            byte_size,
            metadata,
        }
    }

    #[doc = include_str!("resource_compiler.doc/byte_size.md")]
    #[must_use]
    pub const fn byte_size(&self) -> u64 {
        self.byte_size
    }

    #[doc = include_str!("resource_compiler.doc/metadata.md")]
    #[must_use]
    pub fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }
}

#[doc = include_str!("resource_compiler.doc/ResourceCompiler.md")]
pub trait ResourceCompiler {
    #[doc = include_str!("resource_compiler.doc/supports.md")]
    #[must_use]
    fn supports(&self, kind: ResourceKind) -> bool;

    #[doc = include_str!("resource_compiler.doc/compile.md")]
    fn compile(
        &self,
        kind: ResourceKind,
        decoded: &DecodedResource,
    ) -> Result<CompiledResource, ResourceError>;
}

#[doc = include_str!("resource_compiler.doc/NullCompiler.md")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NullCompiler;

impl NullCompiler {
    #[doc = include_str!("resource_compiler.doc/new_2.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ResourceCompiler for NullCompiler {
    fn supports(&self, _kind: ResourceKind) -> bool {
        // Honest: a null compiler handles no kind.
        false
    }

    fn compile(
        &self,
        _kind: ResourceKind,
        _decoded: &DecodedResource,
    ) -> Result<CompiledResource, ResourceError> {
        Err(ResourceError::CompileFailed(
            "no compiler registered".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn sample_decoded() -> DecodedResource {
        DecodedResource::new(
            Arc::from(&[0u8, 1, 2, 3][..]),
            ResourceMetadata::default().with_byte_size(4),
        )
    }

    /// Compile-time proof the port is object-safe: adapters must be storable as
    /// `Box<dyn ResourceCompiler>`.
    #[test]
    fn resource_compiler_is_object_safe() {
        let compiler: Box<dyn ResourceCompiler> = Box::new(NullCompiler::new());
        assert!(!compiler.supports(ResourceKind::Image));
    }

    #[test]
    fn null_compiler_supports_nothing() {
        let compiler = NullCompiler::new();
        for kind in [
            ResourceKind::Image,
            ResourceKind::Font,
            ResourceKind::Stylesheet,
            ResourceKind::Shader,
            ResourceKind::Svg,
            ResourceKind::Atlas,
            ResourceKind::EmbeddedAsset,
        ] {
            assert!(!compiler.supports(kind));
        }
    }

    #[test]
    fn null_compiler_compile_fails_honestly() {
        let compiler = NullCompiler::new();
        assert_eq!(
            compiler.compile(ResourceKind::Image, &sample_decoded()),
            Err(ResourceError::CompileFailed(
                "no compiler registered".into()
            ))
        );
    }

    #[test]
    fn compiled_resource_accessors() {
        let meta = ResourceMetadata::default().with_byte_size(64);
        let compiled = CompiledResource::new(256, meta.clone());
        assert_eq!(compiled.byte_size(), 256);
        assert_eq!(compiled.metadata(), &meta);
    }
}
