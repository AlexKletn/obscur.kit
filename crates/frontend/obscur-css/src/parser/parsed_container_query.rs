use super::{ContainerCondition, ContainerLogic};

/// A parsed `@container` query: an optional name and its feature conditions.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedContainerQuery {
    /// The container name, if the query targets a named container.
    pub name: Option<Box<str>>,
    /// The feature conditions, in source order.
    pub conditions: Vec<ContainerCondition>,
    /// The top-level logical operator.
    pub logic: ContainerLogic,
}
