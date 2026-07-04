//! `UiIrValidationError`: one structural fault found while validating a UI IR
//! document (spec §2.5 validation).

/// A single structural fault discovered while validating a [`UiIrDocument`]
/// (spec §2.5).
///
/// Each variant names one well-formedness rule the IR tree violated. Faults
/// carry enough context to point a diagnostic back at the offending node — most
/// variants embed the stable node id, exposed uniformly via
/// [`node_id`](UiIrValidationError::node_id). The same type is used for both
/// hard errors and non-fatal warnings; whether a given fault is fatal is decided
/// by which bucket it lands in on the [`UiIrValidationResult`], not by the
/// variant itself.
///
/// [`UiIrDocument`]: crate::document::UiIrDocument
/// [`UiIrValidationResult`]: crate::validation::UiIrValidationResult
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiIrValidationError {
    /// Two nodes in the tree share the same stable id; the payload is the
    /// duplicated id. Ids must be unique so source maps and diagnostics resolve
    /// unambiguously.
    DuplicateNodeId(u64),
    /// An `Element` or `Component` node carries an empty tag; the payload is the
    /// node's id. Such nodes have no element name or component to instantiate.
    EmptyElementTag(u64),
    /// A `Text` node has children; the payload is the node's id. Text leaves
    /// cannot contain a subtree.
    TextNodeHasChildren(u64),
    /// A property's value is a binding with an empty path. Names the owning
    /// `node` and the offending `property`.
    EmptyBindingPath {
        /// The stable id of the node owning the property.
        node: u64,
        /// The name of the property whose binding path was empty.
        property: Box<str>,
    },
    /// A required capability was not present in a provided allow-set, or a
    /// declared capability id was itself empty; the payload is the offending
    /// capability id (empty string for the empty-id case).
    UndeclaredCapability(Box<str>),
}

impl UiIrValidationError {
    /// Returns the stable id of the node this fault concerns, if any.
    ///
    /// [`UndeclaredCapability`](UiIrValidationError::UndeclaredCapability)
    /// concerns a capability rather than a node and so returns `None`.
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
