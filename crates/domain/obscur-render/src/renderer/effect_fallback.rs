use super::SceneEffect;

#[doc = include_str!("../../docs/api/renderer/effect_fallback.doc/EffectFallback.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffectFallback {
    effect: SceneEffect,
    reason: Box<str>,
}

impl EffectFallback {
    /// A fallback on `effect`, with the backend's account of what it drew instead.
    #[must_use]
    pub fn new(effect: SceneEffect, reason: impl Into<Box<str>>) -> Self {
        Self {
            effect,
            reason: reason.into(),
        }
    }

    /// The effect that was degraded.
    #[must_use]
    pub const fn effect(&self) -> SceneEffect {
        self.effect
    }

    /// What the backend drew instead of what the scene asked for.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl std::fmt::Display for EffectFallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "degraded effect: {} ({})", self.effect, self.reason)
    }
}
