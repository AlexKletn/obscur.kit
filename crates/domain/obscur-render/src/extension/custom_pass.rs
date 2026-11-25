//! A privileged custom render pass descriptor (spec §5.3).

use super::PassStage;

#[doc = include_str!("../../docs/api/extension/custom_pass.doc/CustomPass.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomPass {
    name: Box<str>,
    stage: PassStage,
    requires_shaders: bool,
}

impl CustomPass {
    /// Creates a custom pass named `name` scheduled at `stage`, using no
    /// restricted user shaders.
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, stage: PassStage) -> Self {
        Self {
            name: name.into(),
            stage,
            requires_shaders: false,
        }
    }

    /// Sets whether this pass uses restricted user shaders, returning the updated
    /// pass (spec §5.3).
    #[must_use]
    pub fn requiring_shaders(mut self, requires_shaders: bool) -> Self {
        self.requires_shaders = requires_shaders;
        self
    }

    /// The pass name, used for diagnostics.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// When this pass runs relative to the ordinary scene.
    #[must_use]
    pub const fn stage(&self) -> PassStage {
        self.stage
    }

    /// Whether this pass uses restricted user shaders (spec §5.3).
    #[must_use]
    pub const fn requires_shaders(&self) -> bool {
        self.requires_shaders
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_defaults_to_no_shaders() {
        let pass = CustomPass::new("bloom", PassStage::AfterScene);
        assert_eq!(pass.name(), "bloom");
        assert_eq!(pass.stage(), PassStage::AfterScene);
        assert!(!pass.requires_shaders());
    }

    #[test]
    fn requiring_shaders_sets_flag() {
        let pass = CustomPass::new("shade", PassStage::Overlay).requiring_shaders(true);
        assert!(pass.requires_shaders());
        assert_eq!(pass.stage(), PassStage::Overlay);
    }
}
