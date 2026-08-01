#![doc = include_str!("validation_error.doc/_module.md")]

#[doc = include_str!("validation_error.doc/UiIrValidationError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiIrValidationError {
    #[doc = include_str!("validation_error.doc/DuplicateNodeId.md")]
    DuplicateNodeId(u64),
    #[doc = include_str!("validation_error.doc/EmptyElementTag.md")]
    EmptyElementTag(u64),
    #[doc = include_str!("validation_error.doc/TextNodeHasChildren.md")]
    TextNodeHasChildren(u64),
    #[doc = include_str!("validation_error.doc/EmptyBindingPath.md")]
    EmptyBindingPath {
        #[doc = include_str!("validation_error.doc/node.md")]
        node: u64,
        #[doc = include_str!("validation_error.doc/property.md")]
        property: Box<str>,
    },
    #[doc = include_str!("validation_error.doc/UndeclaredCapability.md")]
    UndeclaredCapability(Box<str>),
}

impl UiIrValidationError {
    #[doc = include_str!("validation_error.doc/node_id.md")]
    #[must_use]
    pub const fn node_id(&self) -> Option<u64> {
        match self {
            Self::DuplicateNodeId(id)
            | Self::EmptyElementTag(id)
            | Self::TextNodeHasChildren(id)
            | Self::EmptyBindingPath { node: id, .. } => Some(*id),
            Self::UndeclaredCapability(_) => None,
        }
    }
}

impl std::fmt::Display for UiIrValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateNodeId(id) => write!(f, "duplicate node id: {id}"),
            Self::EmptyElementTag(id) => write!(f, "empty element tag on node {id}"),
            Self::TextNodeHasChildren(id) => write!(f, "text node {id} has children"),
            Self::EmptyBindingPath { node, property } => {
                write!(f, "empty binding path on node {node} property {property}")
            }
            Self::UndeclaredCapability(cap) => write!(f, "undeclared capability: {cap}"),
        }
    }
}

impl std::error::Error for UiIrValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_renders_each_variant() {
        assert_eq!(
            UiIrValidationError::DuplicateNodeId(7).to_string(),
            "duplicate node id: 7"
        );
        assert_eq!(
            UiIrValidationError::EmptyElementTag(3).to_string(),
            "empty element tag on node 3"
        );
        assert_eq!(
            UiIrValidationError::TextNodeHasChildren(4).to_string(),
            "text node 4 has children"
        );
        assert_eq!(
            UiIrValidationError::EmptyBindingPath {
                node: 5,
                property: "title".into(),
            }
            .to_string(),
            "empty binding path on node 5 property title"
        );
        assert_eq!(
            UiIrValidationError::UndeclaredCapability("fs.read".into()).to_string(),
            "undeclared capability: fs.read"
        );
    }

    #[test]
    fn usable_as_std_error() {
        let err: &dyn std::error::Error = &UiIrValidationError::DuplicateNodeId(1);
        assert_eq!(err.to_string(), "duplicate node id: 1");
    }

    #[test]
    fn node_id_reports_owning_node_or_none() {
        assert_eq!(UiIrValidationError::DuplicateNodeId(1).node_id(), Some(1));
        assert_eq!(UiIrValidationError::EmptyElementTag(2).node_id(), Some(2));
        assert_eq!(
            UiIrValidationError::TextNodeHasChildren(3).node_id(),
            Some(3)
        );
        assert_eq!(
            UiIrValidationError::EmptyBindingPath {
                node: 4,
                property: "x".into(),
            }
            .node_id(),
            Some(4)
        );
        assert_eq!(
            UiIrValidationError::UndeclaredCapability("c".into()).node_id(),
            None
        );
    }
}
