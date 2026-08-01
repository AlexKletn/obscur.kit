#![doc = include_str!("custom_pass.doc/_module.md")]

#[doc = include_str!("custom_pass.doc/PassStage.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassStage {
    #[doc = include_str!("custom_pass.doc/BeforeScene.md")]
    BeforeScene,
    #[doc = include_str!("custom_pass.doc/AfterScene.md")]
    AfterScene,
    #[doc = include_str!("custom_pass.doc/Overlay.md")]
    Overlay,
}

#[doc = include_str!("custom_pass.doc/CustomPass.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomPass {
    name: Box<str>,
    stage: PassStage,
    requires_shaders: bool,
}

impl CustomPass {
    #[doc = include_str!("custom_pass.doc/new.md")]
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, stage: PassStage) -> Self {
        Self {
            name: name.into(),
            stage,
            requires_shaders: false,
        }
    }

    #[doc = include_str!("custom_pass.doc/requiring_shaders.md")]
    #[must_use]
    pub fn requiring_shaders(mut self, requires_shaders: bool) -> Self {
        self.requires_shaders = requires_shaders;
        self
    }

    #[doc = include_str!("custom_pass.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[doc = include_str!("custom_pass.doc/stage.md")]
    #[must_use]
    pub const fn stage(&self) -> PassStage {
        self.stage
    }

    #[doc = include_str!("custom_pass.doc/requires_shaders.md")]
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

    #[test]
    fn stages_are_distinct() {
        assert_ne!(PassStage::BeforeScene, PassStage::AfterScene);
        assert_ne!(PassStage::AfterScene, PassStage::Overlay);
    }
}
