#![doc = include_str!("timeline_state.doc/_module.md")]

#[doc = include_str!("timeline_state.doc/TimelineState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineState {
    #[doc = include_str!("timeline_state.doc/Idle.md")]
    #[default]
    Idle,
    #[doc = include_str!("timeline_state.doc/Running.md")]
    Running,
    #[doc = include_str!("timeline_state.doc/Paused.md")]
    Paused,
    #[doc = include_str!("timeline_state.doc/Finished.md")]
    Finished,
}

impl TimelineState {
    #[doc = include_str!("timeline_state.doc/is_running.md")]
    #[must_use]
    pub const fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }

    #[doc = include_str!("timeline_state.doc/is_finished.md")]
    #[must_use]
    pub const fn is_finished(self) -> bool {
        matches!(self, Self::Finished)
    }

    #[doc = include_str!("timeline_state.doc/is_active.md")]
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_idle() {
        assert_eq!(TimelineState::default(), TimelineState::Idle);
    }

    #[test]
    fn is_running_only_for_running() {
        assert!(TimelineState::Running.is_running());
        assert!(!TimelineState::Idle.is_running());
        assert!(!TimelineState::Paused.is_running());
        assert!(!TimelineState::Finished.is_running());
    }

    #[test]
    fn is_finished_only_for_finished() {
        assert!(TimelineState::Finished.is_finished());
        assert!(!TimelineState::Running.is_finished());
    }

    #[test]
    fn is_active_for_running_and_paused() {
        assert!(TimelineState::Running.is_active());
        assert!(TimelineState::Paused.is_active());
        assert!(!TimelineState::Idle.is_active());
        assert!(!TimelineState::Finished.is_active());
    }
}
