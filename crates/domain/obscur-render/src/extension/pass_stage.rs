/// When a [`CustomPass`](super::CustomPass) runs relative to the ordinary scene (spec §5.3).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_are_distinct() {
        assert_ne!(PassStage::BeforeScene, PassStage::AfterScene);
        assert_ne!(PassStage::AfterScene, PassStage::Overlay);
    }
}
