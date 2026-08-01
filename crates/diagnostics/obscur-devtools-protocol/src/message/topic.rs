#![doc = include_str!("topic.doc/_module.md")]

#[doc = include_str!("topic.doc/Topic.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Topic {
    #[doc = include_str!("topic.doc/Diagnostics.md")]
    Diagnostics,
    #[doc = include_str!("topic.doc/Lifecycle.md")]
    Lifecycle,
    #[doc = include_str!("topic.doc/Frames.md")]
    Frames,
    #[doc = include_str!("topic.doc/Focus.md")]
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
