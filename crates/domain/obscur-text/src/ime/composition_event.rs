#![doc = include_str!("composition_event.doc/_module.md")]

use super::Preedit;

#[doc = include_str!("composition_event.doc/CompositionEvent.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositionEvent {
    #[doc = include_str!("composition_event.doc/Start.md")]
    Start,
    #[doc = include_str!("composition_event.doc/Update.md")]
    Update(Preedit),
    #[doc = include_str!("composition_event.doc/Commit.md")]
    Commit(Box<str>),
    #[doc = include_str!("composition_event.doc/Cancel.md")]
    Cancel,
}

impl CompositionEvent {
    #[doc = include_str!("composition_event.doc/is_terminal.md")]
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Commit(_) | Self::Cancel)
    }

    #[doc = include_str!("composition_event.doc/preedit.md")]
    #[must_use]
    pub const fn preedit(&self) -> Option<&Preedit> {
        match self {
            Self::Update(preedit) => Some(preedit),
            _ => None,
        }
    }

    #[doc = include_str!("composition_event.doc/committed_text.md")]
    #[must_use]
    pub fn committed_text(&self) -> Option<&str> {
        match self {
            Self::Commit(text) => Some(text),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_terminal_only_for_commit_and_cancel() {
        assert!(!CompositionEvent::Start.is_terminal());
        assert!(!CompositionEvent::Update(Preedit::new("x")).is_terminal());
        assert!(CompositionEvent::Commit(Box::from("done")).is_terminal());
        assert!(CompositionEvent::Cancel.is_terminal());
    }

    #[test]
    fn preedit_accessor() {
        let ev = CompositionEvent::Update(Preedit::new("ab"));
        assert_eq!(ev.preedit().map(Preedit::text), Some("ab"));
        assert!(CompositionEvent::Start.preedit().is_none());
        assert!(CompositionEvent::Cancel.preedit().is_none());
    }

    #[test]
    fn committed_text_accessor() {
        let ev = CompositionEvent::Commit(Box::from("final"));
        assert_eq!(ev.committed_text(), Some("final"));
        assert!(CompositionEvent::Start.committed_text().is_none());
        assert!(
            CompositionEvent::Update(Preedit::new("x"))
                .committed_text()
                .is_none()
        );
    }
}
