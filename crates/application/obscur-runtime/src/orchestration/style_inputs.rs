//! Per-node cascade inputs fed to the style resolver: [`StyleInputs`].

use obscur_identity::{FastMap, InternedSymbol, NodeId};
use obscur_style::cascade::ResolvedDeclarations;
use obscur_style::property::PropertyValue;
use obscur_tree::tree::RetainedTree;

#[doc = include_str!("../../docs/api/orchestration/style_inputs.doc/StyleInputs.md")]
#[derive(Debug, Clone, Default)]
pub struct StyleInputs {
    /// The cascaded declarations for each node that has any.
    cascaded: FastMap<NodeId, ResolvedDeclarations>,
    /// The cascaded custom properties for each node that has any.
    custom: FastMap<NodeId, FastMap<InternedSymbol, PropertyValue>>,
    /// Shared empty declarations returned by reference for absent nodes.
    empty_cascaded: ResolvedDeclarations,
    /// Shared empty custom-property map returned by reference for absent nodes.
    empty_custom: FastMap<InternedSymbol, PropertyValue>,
}

impl StyleInputs {
    /// Creates an empty set of inputs; every node then resolves to its
    /// default/inherited computed style.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the cascaded declarations for `node`, replacing any previous entry.
    pub fn set_cascaded(&mut self, node: NodeId, decls: ResolvedDeclarations) {
        self.cascaded.insert(node, decls);
    }

    #[doc = include_str!("../../docs/api/orchestration/style_inputs.doc/clear_cascaded.md")]
    pub fn clear_cascaded(&mut self, node: NodeId) {
        self.cascaded.remove(&node);
    }

    /// Sets the cascaded custom properties for `node`, replacing any previous
    /// entry.
    pub fn set_custom(&mut self, node: NodeId, custom: FastMap<InternedSymbol, PropertyValue>) {
        self.custom.insert(node, custom);
    }

    #[doc = include_str!("../../docs/api/orchestration/style_inputs.doc/clear_custom.md")]
    pub fn clear_custom(&mut self, node: NodeId) {
        self.custom.remove(&node);
    }

    #[doc = include_str!("../../docs/api/orchestration/style_inputs.doc/retain_live.md")]
    pub(crate) fn retain_live(&mut self, tree: &RetainedTree) {
        self.cascaded.retain(|node, _| tree.contains(*node));
        self.custom.retain(|node, _| tree.contains(*node));
    }

    /// Returns the cascaded declarations for `node`, or a shared empty set when
    /// the node has no inputs.
    #[must_use]
    pub(crate) fn cascaded_for(&self, node: NodeId) -> &ResolvedDeclarations {
        self.cascaded.get(&node).unwrap_or(&self.empty_cascaded)
    }

    /// Returns the cascaded custom properties for `node`, or a shared empty map
    /// when the node has no inputs.
    #[must_use]
    pub(crate) fn custom_for(&self, node: NodeId) -> &FastMap<InternedSymbol, PropertyValue> {
        self.custom.get(&node).unwrap_or(&self.empty_custom)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use obscur_style::property::{PropertyId, PropertyValue};

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn empty_inputs_return_shared_empty_defaults() {
        let inputs = StyleInputs::new();
        assert!(inputs.cascaded_for(node(1)).is_empty());
        assert!(inputs.custom_for(node(1)).is_empty());
    }

    #[test]
    fn set_and_read_cascaded() {
        let mut inputs = StyleInputs::new();
        let mut decls = ResolvedDeclarations::new();
        decls.insert(PropertyId::Opacity, PropertyValue::Number(0.5));
        inputs.set_cascaded(node(7), decls);

        assert_eq!(
            inputs.cascaded_for(node(7)).get(PropertyId::Opacity),
            Some(&PropertyValue::Number(0.5))
        );
        // An unrelated node still gets the shared empty default.
        assert!(inputs.cascaded_for(node(8)).is_empty());
    }

    #[test]
    fn set_and_read_custom() {
        let mut inputs = StyleInputs::new();
        let name = InternedSymbol::from_raw(3);
        let mut custom = HashMap::default();
        custom.insert(name, PropertyValue::Number(2.0));
        inputs.set_custom(node(4), custom);

        assert_eq!(
            inputs.custom_for(node(4)).get(&name),
            Some(&PropertyValue::Number(2.0))
        );
        assert!(inputs.custom_for(node(5)).is_empty());
    }
}
