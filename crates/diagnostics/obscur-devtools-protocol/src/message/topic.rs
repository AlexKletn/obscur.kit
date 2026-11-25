//! Event-stream topic identifiers (spec §7.6).

#[doc = include_str!("../../docs/api/message/topic.doc/Topic.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Topic {
    /// Structured diagnostics (traces, warnings, errors).
    Diagnostics,
    /// Lifecycle transitions of the runtime and its components.
    Lifecycle,
    /// Frame production and commit events.
    Frames,
    /// Focus-change events.
    Focus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topics_are_distinct() {
        assert_ne!(Topic::Diagnostics, Topic::Frames);
        assert_eq!(Topic::Focus, Topic::Focus);
    }
}
