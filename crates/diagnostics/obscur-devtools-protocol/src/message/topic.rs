//! Event-stream topic identifiers (spec §7.6).

/// A subscribable event stream.
///
/// A [`Subscribe`](super::CommandKind::Subscribe) command names one of these,
/// and every [`ProtocolEvent`](super::ProtocolEvent) is tagged with the topic
/// it belongs to.
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
