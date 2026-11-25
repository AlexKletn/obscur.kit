use super::FeedbackEffect;

/// Host port for tactile UI feedback. The adapter applies device capabilities and user policy.
pub trait HapticFeedback {
    /// Submits a cue without waiting for playback. An adapter may suppress the cue according to platform policy.
    fn play(&self, effect: FeedbackEffect);
}
