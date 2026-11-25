/// Logical structure over a media query’s flat feature conditions.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaLogic {
    /// Index into the query’s flat feature list.
    Leaf(usize),
    /// Every child condition must match.
    All(Vec<MediaLogic>),
    /// At least one child condition must match.
    Any(Vec<MediaLogic>),
    /// Negates the child condition.
    Not(Box<MediaLogic>),
}
