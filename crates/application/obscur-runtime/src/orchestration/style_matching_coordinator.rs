//! Matches authored stylesheets against the retained tree to build the
//! per-node cascade inputs: [`StyleMatchingCoordinator`].

use obscur_style::matching::{StyleMatcher, Stylesheet};
use obscur_tree::tree::RetainedTree;

use super::{StyleInputs, TreeElement};

/// The orchestration glue that turns authored [`Stylesheet`]s plus the retained
/// tree into the [`StyleInputs`] the [`StyleCoordinator`](super::StyleCoordinator)
/// threads into the style resolver.
///
/// This is the seam that finally lets authored CSS reach paint: it walks every
/// element node in the tree, wraps it in a [`TreeElement`] cursor, runs the
/// `obscur-style` [`StyleMatcher`] to cascade the matching rules into that
/// node's winning per-property values, and records any non-empty result under
/// the node's id. Non-element nodes (text, synthetic parts, the root grouping
/// node) carry no CSS identity and are skipped.
///
/// The coordinator is zero-sized and stateless: it reimplements no matching or
/// cascade logic — both live entirely in `obscur-style`.
#[derive(Debug, Clone, Copy, Default)]
pub struct StyleMatchingCoordinator;

impl StyleMatchingCoordinator {
    /// Creates a style-matching coordinator.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Matches `sheets` against every element node in `tree` and returns the
    /// resulting [`StyleInputs`].
    ///
    /// `sheets` must be ordered by ascending cascade origin (system-default
    /// first, inline last), the order [`StyleMatcher::match_element`] expects.
    /// For each element node, the matched-and-cascaded
    /// [`ResolvedDeclarations`](obscur_style::cascade::ResolvedDeclarations) is
    /// recorded under the node's id only when it is non-empty, so nodes that
    /// match no rule add no entry and resolve to their default/inherited style.
    #[must_use]
    pub fn build_inputs(&self, tree: &RetainedTree, sheets: &[Stylesheet]) -> StyleInputs {
        let mut inputs = StyleInputs::new();
        let matcher = StyleMatcher::new();
        for (id, node) in tree.iter() {
            if !node.kind().is_element() {
                continue;
            }
            let element = TreeElement::new(tree, id);
            let resolved = matcher.match_element(sheets, &element);
            if !resolved.is_empty() {
                inputs.set_cascaded(id, resolved);
            }
        }
        inputs
    }

    /// Matches a single `sheet` against `tree`, a convenience over
    /// [`build_inputs`](Self::build_inputs) for the one-origin case.
    #[must_use]
    pub fn match_single(&self, tree: &RetainedTree, sheet: &Stylesheet) -> StyleInputs {
        self.build_inputs(tree, std::slice::from_ref(sheet))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_style::cascade::CascadeOrigin;
    use obscur_style::matching::{BlockDeclaration, StyleRule};
    use obscur_style::property::{PropertyId, PropertyValue, StyleColor};
    use obscur_style::selector::{ComplexSelector, CompoundSelector, SelectorList, SimpleSelector};
    use obscur_tree::reconciliation::Reconciler;
    use obscur_view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

    /// A single-class selector list: `.counter`.
    fn class_selectors(name: &str) -> SelectorList {
        SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![SimpleSelector::Class(name.into())]),
            Vec::new(),
        )])
    }

    fn red() -> PropertyValue {
        PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0))
    }

    /// A stylesheet: `.counter { background-color: red }`.
    fn counter_sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![StyleRule::new(
                class_selectors("counter"),
                vec![BlockDeclaration::normal(PropertyId::BackgroundColor, red())],
            )],
        )
    }

    fn reconcile(view: &View) -> RetainedTree {
        let mut tree = RetainedTree::new();
        let _ = Reconciler::new().reconcile_root(&mut tree, view);
        tree
    }

    #[test]
    fn matches_class_and_records_background() {
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into()))),
        ]));
        let tree = reconcile(&view);
        let child = tree.get(tree.root().unwrap()).unwrap().children().to_vec()[0];

        let inputs = StyleMatchingCoordinator::new().match_single(&tree, &counter_sheet());

        assert_eq!(
            inputs.cascaded_for(child).get(PropertyId::BackgroundColor),
            Some(&red()),
            "the .counter node cascaded a red background"
        );
    }

    #[test]
    fn non_matching_node_gets_no_inputs() {
        // The element carries no `counter` class, so nothing matches.
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("other".into()))),
        ]));
        let tree = reconcile(&view);
        let child = tree.get(tree.root().unwrap()).unwrap().children().to_vec()[0];

        let inputs = StyleMatchingCoordinator::new().match_single(&tree, &counter_sheet());
        assert!(
            inputs.cascaded_for(child).is_empty(),
            "a non-matching node resolves to defaults"
        );
    }

    #[test]
    fn empty_sheets_produce_empty_inputs() {
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into()))),
        ]));
        let tree = reconcile(&view);
        let child = tree.get(tree.root().unwrap()).unwrap().children().to_vec()[0];

        let inputs = StyleMatchingCoordinator::new().build_inputs(&tree, &[]);
        assert!(inputs.cascaded_for(child).is_empty());
    }
}
