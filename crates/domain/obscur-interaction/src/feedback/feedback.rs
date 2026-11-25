use std::rc::Rc;

use super::{FeedbackEffect, HapticFeedback, SoundFeedback};

#[doc = include_str!("../../docs/api/feedback/feedback.doc/Feedback.md")]
#[derive(Clone, Default)]
pub struct Feedback {
    haptics: Option<Rc<dyn HapticFeedback>>,
    sound: Option<Rc<dyn SoundFeedback>>,
}

impl Feedback {
    /// Installs the tactile adapter on this handle.
    #[must_use]
    pub fn with_haptics(mut self, adapter: impl HapticFeedback + 'static) -> Self {
        self.haptics = Some(Rc::new(adapter));
        self
    }

    /// Installs the audio adapter on this handle.
    #[must_use]
    pub fn with_sound(mut self, adapter: impl SoundFeedback + 'static) -> Self {
        self.sound = Some(Rc::new(adapter));
        self
    }

    /// Requests tactile feedback if a tactile adapter was installed.
    pub fn haptic(&self, effect: FeedbackEffect) {
        if let Some(adapter) = &self.haptics {
            adapter.play(effect);
        }
    }

    /// Requests audible feedback if an audio adapter was installed.
    pub fn sound(&self, effect: FeedbackEffect) {
        if let Some(adapter) = &self.sound {
            adapter.play(effect);
        }
    }
}
