//! A privileged custom render pass descriptor (spec §5.3).

/// When a [`CustomPass`] runs relative to the ordinary scene (spec §5.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassStage {
    /// Before the ordinary scene is drawn (a background pass).
    BeforeScene,
    /// After the ordinary scene is drawn (a foreground pass).
    AfterScene,
    /// Composited on top as an overlay, above the scene and any after-scene
    /// passes.
    Overlay,
}

/// A descriptor of a privileged custom render pass a trusted extension runs
/// (spec §5.3).
///
/// Trusted, privileged extensions may inject backend-specific passes — for
/// example a GPU post-process or an overlay — scheduled by [`Self::stage`]
/// relative to the ordinary scene. A pass is pure data describing *what* to run
/// and *when*; it does **not** redefine ordinary Canvas semantics (spec §5.3).
/// A pass that uses restricted user shaders sets [`Self::requires_shaders`],
/// which gates a stricter capability check at grant time.
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

    #[test]
    fn stages_are_distinct() {
        assert_ne!(PassStage::BeforeScene, PassStage::AfterScene);
        assert_ne!(PassStage::AfterScene, PassStage::Overlay);
    }
}
