use super::{KeyframeSelector, ParsedDeclaration};

/// One keyframe within a [`ParsedKeyframesRule`](super::ParsedKeyframesRule).
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedKeyframe {
    /// The keyframe selectors (`from`, `to`, or percentage offsets).
    pub selectors: Vec<KeyframeSelector>,
    /// The declarations applied at this keyframe.
    pub declarations: Vec<ParsedDeclaration>,
}
