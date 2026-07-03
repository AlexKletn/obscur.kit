//! A privileged rendering extension gated behind a capability (spec §5.3).

use crate::extension::{CustomPass, TrustedRenderContext};

/// A privileged rendering extension bundling custom passes behind a required
/// capability (spec §5.3).
///
/// Trusted extensions may inject backend-specific [`CustomPass`]es, but only
/// after the host's capability check passes (spec §5.3). This value pairs the
/// passes with the [`Self::required_capability`] the host must have granted and
/// evaluates that check via [`Self::is_permitted`]. It never redefines ordinary
/// Canvas semantics — it only schedules additional privileged work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderExtension {
    name: Box<str>,
    required_capability: Box<str>,
    passes: Vec<CustomPass>,
}

impl RenderExtension {
    /// Creates an extension named `name` that requires the host to have granted
    /// `required_capability`, carrying no passes yet.
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, required_capability: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            required_capability: required_capability.into(),
            passes: Vec::new(),
        }
    }

    /// Appends a custom pass, returning the updated extension.
    #[must_use]
    pub fn with_pass(mut self, pass: CustomPass) -> Self {
        self.passes.push(pass);
        self
    }

    /// The extension name, used for diagnostics.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The capability id the host must have granted for this extension to run
    /// (spec §5.3).
    #[must_use]
    pub fn required_capability(&self) -> &str {
        &self.required_capability
    }

    /// The custom passes this extension contributes.
    #[must_use]
    pub fn passes(&self) -> &[CustomPass] {
        &self.passes
    }

    /// The number of custom passes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.passes.len()
    }

    /// Whether this extension contributes no passes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.passes.is_empty()
    }

    /// Whether `ctx` permits this extension to run — the capability check of
    /// spec §5.3.
    ///
    /// This is a **compound gate**: the context must have granted
    /// [`Self::required_capability`], *and*, if any of the extension's passes
    /// [`requires_shaders`](CustomPass::requires_shaders), the context must also
    /// allow custom shaders
    /// ([`TrustedRenderContext::allows_custom_shaders`]). An extension with no
    /// shader passes needs only the capability grant.
    #[must_use]
    pub fn is_permitted(&self, ctx: &TrustedRenderContext) -> bool {
        if !ctx.has_capability(&self.required_capability) {
            return false;
        }
        let needs_shaders = self.passes.iter().any(CustomPass::requires_shaders);
        !needs_shaders || ctx.allows_custom_shaders()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extension::PassStage;

    #[test]
    fn new_extension_is_empty() {
        let ext = RenderExtension::new("post", "gpu.post");
        assert_eq!(ext.name(), "post");
        assert_eq!(ext.required_capability(), "gpu.post");
        assert!(ext.is_empty());
        assert_eq!(ext.len(), 0);
    }

    #[test]
    fn with_pass_appends() {
        let ext = RenderExtension::new("post", "gpu.post")
            .with_pass(CustomPass::new("a", PassStage::BeforeScene))
            .with_pass(CustomPass::new("b", PassStage::Overlay));
        assert_eq!(ext.len(), 2);
        assert!(!ext.is_empty());
        assert_eq!(ext.passes()[0].name(), "a");
    }

    #[test]
    fn permitted_when_capability_granted_and_no_shaders() {
        let ext = RenderExtension::new("post", "gpu.post")
            .with_pass(CustomPass::new("a", PassStage::AfterScene));
        let ctx = TrustedRenderContext::new().granting("gpu.post");
        assert!(ext.is_permitted(&ctx));
    }

    #[test]
    fn denied_when_capability_missing() {
        let ext = RenderExtension::new("post", "gpu.post");
        let ctx = TrustedRenderContext::new().granting("gpu.other");
        assert!(!ext.is_permitted(&ctx));
    }

    #[test]
    fn shader_pass_requires_custom_shaders_grant() {
        let ext = RenderExtension::new("post", "gpu.post")
            .with_pass(CustomPass::new("shade", PassStage::Overlay).requiring_shaders(true));

        // Capability granted but shaders not allowed: denied.
        let no_shaders = TrustedRenderContext::new().granting("gpu.post");
        assert!(!ext.is_permitted(&no_shaders));

        // Capability granted and shaders allowed: permitted.
        let with_shaders = TrustedRenderContext::new()
            .granting("gpu.post")
            .allow_custom_shaders(true);
        assert!(ext.is_permitted(&with_shaders));
    }
}
