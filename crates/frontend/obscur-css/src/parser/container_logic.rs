use super::ContainerGroup;

/// How the query combines conditions for one selected container.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ContainerLogic {
    #[default]
    /// Require all conditions.
    All,
    /// Require at least one condition.
    Any,
    /// Negate the single condition.
    Not,
    /// Contains the logical structure of a grouped query.
    Nested(Box<ContainerGroup>),
}
