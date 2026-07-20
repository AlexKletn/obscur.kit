//! Matches authored stylesheets against the retained tree to build the
//! per-node cascade inputs: [`StyleMatchingCoordinator`].

use obscur_identity::NodeId;
use obscur_style::cascade::{Cascade, Declaration, ResolvedDeclarations};
use obscur_style::invalidation::StyleInvalidation;
use obscur_style::matching::{MediaEnvironment, StyleMatcher, Stylesheet};
use obscur_style::selector::MatchElement;
use obscur_tree::tree::RetainedTree;

use super::{
    PseudoStates, StyleInputs, TreeElement, component_host, part_specificity_for, subject_part_name,
};

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
        // The stateless path: no interaction state, so `:hover`/`:focus` never
        // match, and an unbounded environment, so every `@media`/`@container`
        // rule applies. This keeps behaviour identical for callers that supply
        // neither interaction state nor a viewport.
        self.build_inputs_with_pseudo(tree, sheets, &PseudoStates::new())
    }

    /// Matches `sheets` against every element node in `tree` in `environment`,
    /// gating each `@media`/`@container` rule by whether its condition holds.
    ///
    /// The stateless (no-interaction) counterpart of
    /// [`build_inputs_with_pseudo_env`](Self::build_inputs_with_pseudo_env): only
    /// the media/container environment participates, so `:hover`/`:focus` still
    /// never match.
    #[must_use]
    pub fn build_inputs_env(
        &self,
        tree: &RetainedTree,
        sheets: &[Stylesheet],
        environment: &MediaEnvironment,
    ) -> StyleInputs {
        self.build_inputs_with_pseudo_env(tree, sheets, &PseudoStates::new(), environment)
    }

    /// Matches `sheets` against every element node in `tree`, sourcing each
    /// node's interaction state from `pseudo`, and returns the resulting
    /// [`StyleInputs`].
    ///
    /// This is the pseudo-state-aware counterpart of
    /// [`build_inputs`](Self::build_inputs): each element is wrapped in a
    /// [`TreeElement::with_pseudo`] cursor so the matcher can honour
    /// interaction-driven pseudo-classes (`:hover`, `:focus`, `:active`, …)
    /// against the state the runtime recorded in `pseudo`. Passing an empty
    /// [`PseudoStates`] is exactly equivalent to
    /// [`build_inputs`](Self::build_inputs). `sheets` ordering and the
    /// non-empty-only recording rule are identical to
    /// [`build_inputs`](Self::build_inputs).
    #[must_use]
    pub fn build_inputs_with_pseudo(
        &self,
        tree: &RetainedTree,
        sheets: &[Stylesheet],
        pseudo: &PseudoStates,
    ) -> StyleInputs {
        self.build_inputs_with_pseudo_env(tree, sheets, pseudo, &MediaEnvironment::unbounded())
    }

    /// Matches `sheets` against every element node in `tree`, honouring both the
    /// interaction state in `pseudo` and the media/container `environment`.
    ///
    /// This is the fully-parameterised worker the runtime frame path uses: each
    /// element is matched with [`StyleMatcher::match_element_in`], so a rule that
    /// was authored inside an `@media`/`@container` group contributes only when
    /// its condition holds against `environment` (§4.13), while `:hover`/`:focus`
    /// are honoured against `pseudo`. Passing [`MediaEnvironment::unbounded`] is
    /// exactly equivalent to [`build_inputs_with_pseudo`](Self::build_inputs_with_pseudo).
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
        }
        inputs
    }

    /// Re-matches **only** the nodes `dirty` records, updating `inputs` in place
    /// (spec §3.4 phase 5, §4.3).
    ///
    /// This is the incremental counterpart of
    /// [`build_inputs_with_pseudo_env`](Self::build_inputs_with_pseudo_env): where
    /// that runs every rule against every element, this runs them against the
    /// invalidated elements and leaves every other node's retained entry alone.
    /// The caller owes the precondition — `inputs` must be what the *same* sheets
    /// and `environment` produced over the *same* tree, so that a node outside
    /// `dirty` provably still matches what it matched then. That is the whole
    /// contract: the accumulator says what changed, and only that is recomputed.
    ///
    /// The re-match is a full re-match of each dirty node, not a patch: a node that
    /// matched a rule last frame and matches none now has its entry
    /// [cleared](StyleInputs::clear_cascaded), so a stale cascade cannot survive a
    /// `:hover` ending.
    ///
    /// Returns the number of elements re-matched. Non-element and stale nodes carry
    /// no CSS identity, so they are skipped exactly as the full pass skips them.
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
            matched += 1;
        }
        matched
    }

    /// Matches a single `sheet` against `tree`, a convenience over
    /// [`build_inputs`](Self::build_inputs) for the one-origin case.
    #[must_use]
    pub fn match_single(&self, tree: &RetainedTree, sheet: &Stylesheet) -> StyleInputs {
        self.build_inputs(tree, std::slice::from_ref(sheet))
    }
}

/// Resolves `node`'s cascaded declarations, honouring the component-scope
/// boundary (§4.15).
///
/// A **top-level** node (not inside any component's render output) matches the
/// authored sheets exactly as before — the pure matcher already treats a
/// `::part()` selector as inert, so nothing about the ordinary path changes. A
/// **component-internal** node is *scoped by default*: the outside sheets' plain
/// selectors do not reach it, and only an exported style part
/// (`Component::part(name)` targeting a class the internal element carries, with
/// the `Component` half matching the host) crosses the boundary.
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

/// The cascade for a component-internal `node`: only `::part()` rules the host
/// exports cross the scope boundary; every ordinary selector is contained.
///
/// The `source_order` counter advances over **every** encountered declaration
/// (matching or not), exactly as [`StyleMatcher::match_element_in`] does, so
/// document-order tie-breaks are identical for the declarations that do cross.
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
        for rule in sheet.rules() {
            // The one door through the boundary: a `::part(name)` selector whose
            // exported name is a class this internal element carries and whose
            // host half matches the component host. Take the highest specificity
            // among the list's crossing selectors.
            let specificity = rule
                .selectors()
                .selectors()
                .iter()
                .filter_map(|selector| {
                    let want = subject_part_name(selector)?;
                    element
                        .has_class(want)
                        .then(|| part_specificity_for(selector, want, &host_cursor))
                        .flatten()
                })
                .max();
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
