//! Per-node cascade inputs fed to the style resolver: [`StyleInputs`].

use std::collections::HashMap;

use obscur_identity::{InternedSymbol, NodeId};
use obscur_style::cascade::ResolvedDeclarations;
use obscur_style::property::PropertyValue;

/// The already-cascaded style inputs the [`StyleCoordinator`] threads into the
/// resolver, keyed by [`NodeId`].
///
/// # Phase-1 stand-in
///
/// Matching a stylesheet against the tree and running the cascade is the job of
/// the `obscur-css` crate, which does not exist yet. Until it does, Phase 1
/// supplies the **already-cascaded** per-node values directly: a
/// [`ResolvedDeclarations`] (the winning cascaded value per property) plus the
/// node's cascaded custom properties. The [`StyleCoordinator`] never runs a
/// cascade itself — it only walks the tree and hands these inputs to
/// `obscur-style` (spec §3.10).
///
/// An unset node resolves to its default/inherited computed style: the getters
/// return shared empty inputs by reference when a node has no entry, so every
/// node in the tree can be resolved whether or not it was given inputs.
///
/// [`StyleCoordinator`]: super::StyleCoordinator
#[derive(Debug, Clone, Default)]
pub struct StyleInputs {
    /// The cascaded declarations for each node that has any.
    cascaded: HashMap<NodeId, ResolvedDeclarations>,
    /// The cascaded custom properties for each node that has any.
    custom: HashMap<NodeId, HashMap<InternedSymbol, PropertyValue>>,
    /// Shared empty declarations returned by reference for absent nodes.
    empty_cascaded: ResolvedDeclarations,
    /// Shared empty custom-property map returned by reference for absent nodes.
    empty_custom: HashMap<InternedSymbol, PropertyValue>,
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

    /// Drops `node`'s cascaded declarations, so it resolves to its
    /// default/inherited computed style again.
    ///
    /// This is the counterpart [`set_cascaded`](Self::set_cascaded) needs once
    /// inputs are rebuilt for *some* nodes rather than from scratch: a node that
    /// matched a rule last frame and matches none this frame must lose its entry,
    /// not keep the stale one.
    pub fn clear_cascaded(&mut self, node: NodeId) {
        self.cascaded.remove(&node);
    }

    /// Sets the cascaded custom properties for `node`, replacing any previous
    /// entry.
    pub fn set_custom(&mut self, node: NodeId, custom: HashMap<InternedSymbol, PropertyValue>) {
        self.custom.insert(node, custom);
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
    pub(crate) fn custom_for(&self, node: NodeId) -> &HashMap<InternedSymbol, PropertyValue> {
        self.custom.get(&node).unwrap_or(&self.empty_custom)
    }
}

#[cfg(test)]
mod tests {
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
        let mut custom = HashMap::new();
        custom.insert(name, PropertyValue::Number(2.0));
        inputs.set_custom(node(4), custom);

        assert_eq!(
            inputs.custom_for(node(4)).get(&name),
            Some(&PropertyValue::Number(2.0))
        );
        assert!(inputs.custom_for(node(5)).is_empty());
    }
}
