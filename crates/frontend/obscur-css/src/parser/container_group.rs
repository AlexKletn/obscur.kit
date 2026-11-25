use super::ContainerLogic;

/// A logical expression over the leaves of a parsed query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerGroup {
    /// Indexes a leaf in the parsed query’s condition list.
    Leaf(usize),
    /// The indices of the two comparisons in one chained style range.
    Range([usize; 2]),
    /// Require all conditions.
    All(Vec<Self>),
    /// Require at least one condition.
    Any(Vec<Self>),
    /// Negate the single condition.
    Not(Box<Self>),
}

impl ContainerGroup {
    pub(super) fn into_logic(self) -> ContainerLogic {
        let all_leaves =
            |children: &[Self]| children.iter().all(|child| matches!(child, Self::Leaf(_)));
        match self {
            Self::Leaf(_) => ContainerLogic::All,
            Self::All(ref children) if all_leaves(children) => ContainerLogic::All,
            Self::Any(ref children) if all_leaves(children) => ContainerLogic::Any,
            Self::Not(ref child) if matches!(**child, Self::Leaf(_)) => ContainerLogic::Not,
            group => ContainerLogic::Nested(Box::new(group)),
        }
    }
}
