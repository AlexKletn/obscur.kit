//! Platform-neutral requests for tactile and audible UI feedback.

mod feedback;
pub use feedback::Feedback;
mod feedback_effect;
pub use feedback_effect::FeedbackEffect;
mod haptic_feedback;
pub use haptic_feedback::HapticFeedback;
mod sound_feedback;
pub use sound_feedback::SoundFeedback;
