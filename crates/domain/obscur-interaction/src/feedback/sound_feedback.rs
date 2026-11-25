use super::FeedbackEffect;

/// Host port for audible UI feedback. The adapter applies device capabilities and user policy.
pub trait SoundFeedback {
    /// Submits a cue without waiting for playback. An adapter may suppress the cue according to platform policy.
    fn play(&self, effect: FeedbackEffect);
}
