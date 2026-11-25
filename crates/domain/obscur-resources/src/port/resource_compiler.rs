#![doc = include_str!("../../docs/api/port/resource_compiler.doc/_module.md")]

use crate::handle::ResourceKind;
use crate::state::ResourceError;

use super::{CompiledResource, DecodedResource};

#[doc = include_str!("../../docs/api/port/resource_compiler.doc/ResourceCompiler.md")]
pub trait ResourceCompiler {
    /// Returns whether this compiler can compile resources of `kind` (spec
    /// §5.15).
    #[must_use]
    fn supports(&self, kind: ResourceKind) -> bool;

    #[doc = include_str!("../../docs/api/port/resource_compiler.doc/compile.md")]
    fn compile(
        &self,
        kind: ResourceKind,
        decoded: &DecodedResource,
    ) -> Result<CompiledResource, ResourceError>;
}

#[cfg(test)]
mod tests {
    use super::super::NullCompiler;
    use super::*;

    /// Compile-time proof the port is object-safe: adapters must be storable as
    /// `Box<dyn ResourceCompiler>`.
    #[test]
    fn resource_compiler_is_object_safe() {
        let compiler: Box<dyn ResourceCompiler> = Box::new(NullCompiler::new());
        assert!(!compiler.supports(ResourceKind::Image));
    }
}
