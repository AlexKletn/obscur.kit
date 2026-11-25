use crate::handle::ResourceKind;
use crate::state::ResourceError;

use super::{CompiledResource, DecodedResource, ResourceCompiler};

#[doc = include_str!("../../docs/api/port/null_compiler.doc/NullCompiler.md")]
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
        Err(ResourceError::CompileFailed(
            "no compiler registered".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::state::ResourceMetadata;

    fn sample_decoded() -> DecodedResource {
        DecodedResource::new(
            Arc::from(&[0u8, 1, 2, 3][..]),
            ResourceMetadata::default().with_byte_size(4),
        )
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
}
