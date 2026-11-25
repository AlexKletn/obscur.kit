//! The capability-checked grant a trusted extension holds (spec §5.3).

use std::collections::BTreeSet;

#[doc = include_str!("../../docs/api/extension/trusted_render_context.doc/TrustedRenderContext.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedRenderContext {
    granted_capabilities: BTreeSet<Box<str>>,
    allow_custom_shaders: bool,
    allow_gpu_resources: bool,
}

impl TrustedRenderContext {
    /// Creates an untrusted context: no granted capabilities and neither custom
    /// shaders nor GPU resources allowed.
    #[must_use]
    pub fn new() -> Self {
        Self {
            granted_capabilities: BTreeSet::new(),
            allow_custom_shaders: false,
            allow_gpu_resources: false,
        }
    }

    /// Adds a granted capability `id`, returning the updated context.
    #[must_use]
    pub fn granting(mut self, id: impl Into<Box<str>>) -> Self {
        self.granted_capabilities.insert(id.into());
        self
    }

    /// Sets whether custom shaders are allowed, returning the updated context
    /// (spec §5.3).
    #[must_use]
    pub fn allow_custom_shaders(mut self, allow: bool) -> Self {
        self.allow_custom_shaders = allow;
        self
    }

    /// Sets whether raw GPU resources are allowed, returning the updated context
    /// (spec §5.3).
    #[must_use]
    pub fn allow_gpu_resources(mut self, allow: bool) -> Self {
        self.allow_gpu_resources = allow;
        self
    }

    /// Whether the host granted the capability named `id`.
    #[must_use]
    pub fn has_capability(&self, id: &str) -> bool {
        self.granted_capabilities.contains(id)
    }

    /// The set of granted capability ids.
    #[must_use]
    pub const fn granted_capabilities(&self) -> &BTreeSet<Box<str>> {
        &self.granted_capabilities
    }

    /// Whether custom shaders are allowed (spec §5.3).
    #[must_use]
    pub const fn allows_custom_shaders(&self) -> bool {
        self.allow_custom_shaders
    }

    /// Whether raw GPU resources are allowed (spec §5.3).
    #[must_use]
    pub const fn allows_gpu_resources(&self) -> bool {
        self.allow_gpu_resources
    }

    #[doc = include_str!("../../docs/api/extension/trusted_render_context.doc/is_trusted.md")]
    #[must_use]
    pub fn is_trusted(&self) -> bool {
        !self.granted_capabilities.is_empty()
            || self.allow_custom_shaders
            || self.allow_gpu_resources
    }
}

impl Default for TrustedRenderContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_untrusted() {
        let ctx = TrustedRenderContext::default();
        assert!(!ctx.is_trusted());
        assert!(!ctx.has_capability("gpu.compute"));
        assert!(!ctx.allows_custom_shaders());
        assert!(!ctx.allows_gpu_resources());
        assert!(ctx.granted_capabilities().is_empty());
    }

    #[test]
    fn granting_records_capabilities() {
        let ctx = TrustedRenderContext::new()
            .granting("gpu.compute")
            .granting("gpu.texture");
        assert!(ctx.has_capability("gpu.compute"));
        assert!(ctx.has_capability("gpu.texture"));
        assert!(!ctx.has_capability("gpu.raytrace"));
        assert!(ctx.is_trusted());
    }

    #[test]
    fn allow_flags_confer_trust() {
        assert!(
            TrustedRenderContext::new()
                .allow_custom_shaders(true)
                .is_trusted()
        );
        assert!(
            TrustedRenderContext::new()
                .allow_gpu_resources(true)
                .is_trusted()
        );
    }
}
