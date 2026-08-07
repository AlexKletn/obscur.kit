#![doc = include_str!("style_matching_coordinator.doc/_module.md")]

use std::collections::HashMap;

use obscur_identity::{InternedSymbol, NodeId};
use obscur_style::cascade::{Cascade, CascadeOrigin, Declaration, ResolvedDeclarations};
use obscur_style::invalidation::StyleInvalidation;
use obscur_style::matching::{MediaEnvironment, StyleMatcher, Stylesheet};
use obscur_style::property::PropertyValue;
use obscur_style::selector::{MatchElement, SelectorMatcher};
use obscur_tree::tree::RetainedTree;

use super::{
    PseudoStates, StyleInputs, TreeElement, component_host, part_specificity_for, subject_part_name,
};

#[doc = include_str!("style_matching_coordinator.doc/StyleMatchingCoordinator.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct StyleMatchingCoordinator;

impl StyleMatchingCoordinator {
    #[doc = include_str!("style_matching_coordinator.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("style_matching_coordinator.doc/build_inputs.md")]
    #[must_use]
    pub fn build_inputs(&self, tree: &RetainedTree, sheets: &[Stylesheet]) -> StyleInputs {
        // The stateless path: no interaction state, so `:hover`/`:focus` never
        // match, and an unbounded environment, so every `@media`/`@container`
        // rule applies. This keeps behaviour identical for callers that supply
        // neither interaction state nor a viewport.
        self.build_inputs_with_pseudo(tree, sheets, &PseudoStates::new())
    }

    #[doc = include_str!("style_matching_coordinator.doc/build_inputs_env.md")]
    #[must_use]
    pub fn build_inputs_env(
        &self,
        tree: &RetainedTree,
        sheets: &[Stylesheet],
        environment: &MediaEnvironment,
    ) -> StyleInputs {
        self.build_inputs_with_pseudo_env(tree, sheets, &PseudoStates::new(), environment)
    }

    #[doc = include_str!("style_matching_coordinator.doc/build_inputs_with_pseudo.md")]
    #[must_use]
    pub fn build_inputs_with_pseudo(
        &self,
        tree: &RetainedTree,
        sheets: &[Stylesheet],
        pseudo: &PseudoStates,
    ) -> StyleInputs {
        self.build_inputs_with_pseudo_env(tree, sheets, pseudo, &MediaEnvironment::unbounded())
    }

    #[doc = include_str!("style_matching_coordinator.doc/build_inputs_with_pseudo_env.md")]
    #[must_use]
    pub fn build_inputs_with_pseudo_env(
        &self,
        tree: &RetainedTree,
        sheets: &[Stylesheet],
        pseudo: &PseudoStates,
        environment: &MediaEnvironment,
    ) -> StyleInputs {
        let mut inputs = StyleInputs::new();
        let matcher = StyleMatcher::new();
        for (id, node) in tree.iter() {
            if !node.kind().is_element() {
                continue;
            }
            let resolved = resolved_for_node(&matcher, sheets, tree, id, pseudo, environment);
            if !resolved.is_empty() {
                inputs.set_cascaded(id, resolved);
            }
            let custom = custom_for_node(matcher, sheets, tree, id, pseudo, environment);
            if !custom.is_empty() {
                inputs.set_custom(id, custom);
            }
        }
        inputs
    }

    #[doc = include_str!("style_matching_coordinator.doc/rematch_into.md")]
    pub fn rematch_into(
        &self,
        inputs: &mut StyleInputs,
        tree: &RetainedTree,
        sheets: &[Stylesheet],
        pseudo: &PseudoStates,
        environment: &MediaEnvironment,
        dirty: &StyleInvalidation,
    ) -> usize {
        let matcher = StyleMatcher::new();
        let mut matched = 0;
        for (&id, _) in dirty {
            if !tree.get(id).is_some_and(|node| node.kind().is_element()) {
                continue;
            }
            let resolved = resolved_for_node(&matcher, sheets, tree, id, pseudo, environment);
            if resolved.is_empty() {
                inputs.clear_cascaded(id);
            } else {
                inputs.set_cascaded(id, resolved);
            }
            // Rematched with the declarations, not left behind: a node that
            // stopped matching `.dark` must stop carrying `.dark`'s `--brand`,
            // and the whole subtree below it inherits the difference.
            let custom = custom_for_node(matcher, sheets, tree, id, pseudo, environment);
            if custom.is_empty() {
                inputs.clear_custom(id);
            } else {
                inputs.set_custom(id, custom);
            }
            matched += 1;
        }
        matched
    }

    #[doc = include_str!("style_matching_coordinator.doc/match_single.md")]
    #[must_use]
    pub fn match_single(&self, tree: &RetainedTree, sheet: &Stylesheet) -> StyleInputs {
        self.build_inputs(tree, std::slice::from_ref(sheet))
    }

    #[doc = include_str!("style_matching_coordinator.doc/collect_invalid_value_faults.md")]
    #[must_use]
    pub fn collect_invalid_value_faults(
        &self,
        tree: &RetainedTree,
        sheets: &[Stylesheet],
        pseudo: &PseudoStates,
        environment: &MediaEnvironment,
    ) -> Vec<NodeId> {
        let mut faults = Vec::new();
        for (id, node) in tree.iter() {
            if !node.kind().is_element() || component_host(tree, id).is_some() {
                continue;
            }
            let element = TreeElement::with_pseudo(tree, id, pseudo);
            for sheet in sheets {
                for rule in sheet.rules() {
                    if rule.invalid_values().is_empty() {
                        continue;
                    }
                    let condition_holds = rule
                        .condition()
                        .is_none_or(|condition| condition.holds(environment));
                    if condition_holds
                        && SelectorMatcher::matches_list(rule.selectors(), &element).is_some()
                    {
                        // One fault per dropped declaration this element matched.
                        faults.extend(rule.invalid_values().iter().map(|_| id));
                    }
                }
            }
        }
        faults
    }
}

#[doc = include_str!("style_matching_coordinator.doc/resolved_for_node.md")]
fn resolved_for_node(
    matcher: &StyleMatcher,
    sheets: &[Stylesheet],
    tree: &RetainedTree,
    node: NodeId,
    pseudo: &PseudoStates,
    environment: &MediaEnvironment,
) -> ResolvedDeclarations {
    let element = TreeElement::with_pseudo(tree, node, pseudo);
    match component_host(tree, node) {
        None => matcher.match_element_in(sheets, &element, environment),
        Some(host) => scoped_internal_declarations(sheets, tree, node, host, pseudo, environment),
    }
}

#[doc = include_str!("style_matching_coordinator.doc/custom_for_node.md")]
fn custom_for_node(
    matcher: StyleMatcher,
    sheets: &[Stylesheet],
    tree: &RetainedTree,
    node: NodeId,
    pseudo: &PseudoStates,
    environment: &MediaEnvironment,
) -> HashMap<InternedSymbol, PropertyValue> {
    // A component's internals declare none of their own: an app sheet's
    // `.card { --pad: 8px }` must not reach inside a component that happens to
    // use the class, exactly as its longhands do not. They still SEE the host's
    // custom properties — those inherit down the computed tree, which is how a
    // component is themed from outside without opening its boundary.
    if component_host(tree, node).is_some() {
        return HashMap::new();
    }
    let element = TreeElement::with_pseudo(tree, node, pseudo);
    matcher.match_custom_properties_in(sheets, &element, environment)
}

#[doc = include_str!("style_matching_coordinator.doc/scoped_internal_declarations.md")]
fn scoped_internal_declarations(
    sheets: &[Stylesheet],
    tree: &RetainedTree,
    node: NodeId,
    host: NodeId,
    pseudo: &PseudoStates,
    environment: &MediaEnvironment,
) -> ResolvedDeclarations {
    let element = TreeElement::with_pseudo(tree, node, pseudo);
    let host_cursor = TreeElement::with_pseudo(tree, host, pseudo);
    let mut collected: Vec<Declaration> = Vec::new();
    let mut source_order: u32 = 0;
    for sheet in sheets {
        let origin = sheet.origin();
        // The built-in definition of the base elements is not an author sheet and
        // is not scoped: it matches an internal element exactly as it matches an
        // authored one (see the doc comment above).
        let is_system_default = origin == CascadeOrigin::SystemDefault;
        for rule in sheet.rules() {
            // The one door through the boundary: a `::part(name)` selector whose
            // exported name is a class this internal element carries and whose
            // host half matches the component host. Take the highest specificity
            // among the list's crossing selectors.
            let specificity = if is_system_default {
                SelectorMatcher::matches_list(rule.selectors(), &element)
            } else {
                rule.selectors()
                    .selectors()
                    .iter()
                    .filter_map(|selector| {
                        let want = subject_part_name(selector)?;
                        element
                            .has_class(want)
                            .then(|| part_specificity_for(selector, want, &host_cursor))
                            .flatten()
                    })
                    .max()
            };
            let condition_holds = rule
                .condition()
                .is_none_or(|condition| condition.holds(environment));
            for block_decl in rule.block() {
                let order = source_order;
                source_order = source_order.wrapping_add(1);
                if condition_holds && let Some(specificity) = specificity {
                    collected.push(Declaration::new(
                        block_decl.property(),
                        block_decl.value().clone(),
                        block_decl.important_flag(),
                        origin,
                        specificity,
                        order,
                        None,
                    ));
                }
            }
        }
    }
    Cascade::new().resolve(&collected)
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
        let child = tree
            .get(tree.root().unwrap())
            .unwrap()
            .child_nodes()
            .to_vec()[0];

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
        let child = tree
            .get(tree.root().unwrap())
            .unwrap()
            .child_nodes()
            .to_vec()[0];

        let inputs = StyleMatchingCoordinator::new().match_single(&tree, &counter_sheet());
        assert!(
            inputs.cascaded_for(child).is_empty(),
            "a non-matching node resolves to defaults"
        );
    }

    /// `.btn:hover { background-color: red }` — a hover-gated rule on `.btn`.
    fn hover_sheet() -> Stylesheet {
        use obscur_style::selector::PseudoClassSelector;
        let selectors = SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![
                SimpleSelector::Class("btn".into()),
                SimpleSelector::PseudoClass(PseudoClassSelector::Hover),
            ]),
            Vec::new(),
        )]);
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![StyleRule::new(
                selectors,
                vec![BlockDeclaration::normal(PropertyId::BackgroundColor, red())],
            )],
        )
    }

    #[test]
    fn hover_rule_needs_recorded_hover_state() {
        use crate::orchestration::PseudoStates;
        use obscur_style::selector::PseudoState;

        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("btn".into()))),
        ]));
        let tree = reconcile(&view);
        let child = tree
            .get(tree.root().unwrap())
            .unwrap()
            .child_nodes()
            .to_vec()[0];
        let coordinator = StyleMatchingCoordinator::new();
        let sheet = hover_sheet();

        // With no interaction state the hover-gated rule does NOT contribute.
        let empty = PseudoStates::new();
        let cold =
            coordinator.build_inputs_with_pseudo(&tree, std::slice::from_ref(&sheet), &empty);
        assert!(
            cold.cascaded_for(child).is_empty(),
            ":hover must not match without recorded hover state"
        );
        // The stateless convenience agrees.
        assert!(
            coordinator
                .build_inputs(&tree, std::slice::from_ref(&sheet))
                .cascaded_for(child)
                .is_empty()
        );

        // Recording HOVER for the node makes the rule match and cascade.
        let mut hot_states = PseudoStates::new();
        hot_states.set(child, PseudoState::HOVER);
        let hot =
            coordinator.build_inputs_with_pseudo(&tree, std::slice::from_ref(&sheet), &hot_states);
        assert_eq!(
            hot.cascaded_for(child).get(PropertyId::BackgroundColor),
            Some(&red()),
            ":hover matches once hover state is recorded"
        );
    }

    /// A stylesheet: `.counter { --brand: red }`.
    fn brand_sheet() -> Stylesheet {
        use obscur_style::property::CustomProperty;
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![
                StyleRule::new(class_selectors("counter"), Vec::new()).with_custom_properties(
                    vec![CustomProperty::new(InternedSymbol::named("--brand"), red())],
                ),
            ],
        )
    }

    #[test]
    fn a_custom_property_a_rule_declares_reaches_the_node_that_matched_it() {
        // The link this whole path exists for: a compiled `--brand` that never
        // reaches `StyleInputs` is a `var(--brand)` that resolves to nothing,
        // silently, in every sheet that uses one.
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into()))),
        ]));
        let tree = reconcile(&view);
        let child = tree
            .get(tree.root().unwrap())
            .unwrap()
            .child_nodes()
            .to_vec()[0];

        let inputs = StyleMatchingCoordinator::new().match_single(&tree, &brand_sheet());

        assert_eq!(
            inputs
                .custom_for(child)
                .get(&InternedSymbol::named("--brand")),
            Some(&red())
        );
        // The rule declared no longhand, so nothing entered the ordinary cascade.
        assert!(inputs.cascaded_for(child).is_empty());
    }

    #[test]
    fn a_node_that_no_longer_matches_loses_the_custom_property_it_declared() {
        // The incremental path: a stale `--brand` would keep theming a subtree
        // that stopped asking for it.
        use obscur_style::invalidation::StyleInvalidation;

        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into()))),
        ]));
        let tree = reconcile(&view);
        let child = tree
            .get(tree.root().unwrap())
            .unwrap()
            .child_nodes()
            .to_vec()[0];

        let coordinator = StyleMatchingCoordinator::new();
        let mut inputs = coordinator.match_single(&tree, &brand_sheet());
        assert!(!inputs.custom_for(child).is_empty());

        let mut dirty = StyleInvalidation::new();
        dirty.invalidate(
            child,
            obscur_style::invalidation::StyleInvalidationReason::StylesheetChanged,
        );
        coordinator.rematch_into(
            &mut inputs,
            &tree,
            &[],
            &PseudoStates::new(),
            &MediaEnvironment::unbounded(),
            &dirty,
        );
        assert!(
            inputs.custom_for(child).is_empty(),
            "no sheet declares it any more"
        );
    }

    #[test]
    fn empty_sheets_produce_empty_inputs() {
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into()))),
        ]));
        let tree = reconcile(&view);
        let child = tree
            .get(tree.root().unwrap())
            .unwrap()
            .child_nodes()
            .to_vec()[0];

        let inputs = StyleMatchingCoordinator::new().build_inputs(&tree, &[]);
        assert!(inputs.cascaded_for(child).is_empty());
    }
}
