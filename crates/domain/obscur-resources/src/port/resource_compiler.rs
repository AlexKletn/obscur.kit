//! The hexagonal port for compiling a CPU representation into a backend/GPU
//! artifact (spec §5.15).
//!
//! This module defines [`ResourceCompiler`], the port that turns a decoded CPU
//! representation into a backend/GPU artifact ready for the runtime and its
//! backend/GPU cache (spec §5.15 compile). Because the GPU artifact itself is
//! adapter-specific — a texture, an uploaded font atlas, a compiled shader
//! pipeline — the domain never holds it directly: the compiler returns the
//! colocated [`CompiledResource`] descriptor, which records the artifact's
//! backend footprint (for cache budgeting) and its [`ResourceMetadata`]. The
//! colocated [`NullCompiler`] is the honest default that compiles nothing.
//!
//! Compilation is modeled **synchronously** at the domain level: an adapter that
//! uploads on a render thread wraps its own async and calls this port with the
//! result.

use crate::handle::ResourceKind;
use crate::port::DecodedResource;
use crate::state::{ResourceError, ResourceMetadata};

/// A descriptor of a backend/GPU artifact compiled from a resource (spec §5.15).
///
/// This is the output of [`ResourceCompiler::compile`]. The compiled artifact
/// (texture, pipeline, uploaded atlas) lives inside the adapter that produced it;
/// the domain keeps only this opaque descriptor, recording the artifact's
/// backend/GPU `byte_size` — the footprint used for backend-cache budgeting (spec
/// §5.15) — alongside the [`ResourceMetadata`] carried through the pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledResource {
    byte_size: u64,
    metadata: ResourceMetadata,
}

impl CompiledResource {
    /// Creates a compiled-resource descriptor from the artifact's backend/GPU
    /// `byte_size` and its `metadata`.
    #[must_use]
    pub fn new(byte_size: u64, metadata: ResourceMetadata) -> Self {
        Self {
            byte_size,
            metadata,
        }
    }

    /// Returns the backend/GPU footprint in bytes, used for cache budgeting (spec
    /// §5.15).
    #[must_use]
    pub const fn byte_size(&self) -> u64 {
        self.byte_size
    }

    /// Returns the metadata carried through compilation (spec §5.15).
    #[must_use]
    pub fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }
}

/// The hexagonal port for compiling a CPU representation into a backend/GPU
/// artifact (spec §5.15 compile, backend/GPU cache).
///
/// An adapter implements this once per backend, and the domain stores it as
/// `Box<dyn ResourceCompiler>`; the trait is therefore deliberately
/// **object-safe** — no generic methods and no `Self`-returning methods — so
/// compilers can be held polymorphically. A compiler reports failure through
/// [`ResourceError`] rather than panicking.
pub trait ResourceCompiler {
    /// Returns whether this compiler can compile resources of `kind` (spec
    /// §5.15).
    #[must_use]
    fn supports(&self, kind: ResourceKind) -> bool;

    /// Compiles `decoded` of the given `kind` into a backend/GPU artifact (spec
    /// §5.15), returning its [`CompiledResource`] descriptor.
    ///
    /// # Errors
    ///
    /// Returns a [`ResourceError`] on failure — [`ResourceError::Unsupported`]
    /// for a kind this compiler does not handle, or
    /// [`ResourceError::CompileFailed`] when compilation itself fails.
    fn compile(
        &self,
        kind: ResourceKind,
        decoded: &DecodedResource,
    ) -> Result<CompiledResource, ResourceError>;
}

/// An honest reference [`ResourceCompiler`] that compiles nothing (spec §5.15).
///
/// Adapters provide the real compilers that upload textures or build pipelines;
/// `NullCompiler` is the honest default a hub uses before any compiler is
/// registered. It reports [`ResourceCompiler::supports`] `== false` for every
/// [`ResourceKind`] and returns [`ResourceError::CompileFailed`] from
/// [`ResourceCompiler::compile`]. Its refusals are correct behavior, not stubs
/// waiting to be filled in.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NullCompiler;

impl NullCompiler {
    /// Creates a [`NullCompiler`].
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
        Err(ResourceError::CompileFailed("no compiler registered".into()))
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
            Err(ResourceError::CompileFailed("no compiler registered".into()))
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
