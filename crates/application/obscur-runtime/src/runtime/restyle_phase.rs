//! The frame's match + style phase: [`RestylePhase`].

use obscur_diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSink, DiagnosticSubsystem,
    RecoveryAction,
};
use obscur_geometry::Size;
use obscur_identity::NodeId;
use obscur_style::invalidation::StyleInvalidation;
use obscur_style::matching::{MediaEnvironment, Stylesheet};
use obscur_style::property::EnvironmentValues;
use obscur_text::shaping::ParagraphShaper;
use obscur_tree::reconciliation::{ReconciliationOp, ReconciliationPlan};
use obscur_tree::tree::RetainedTree;

use super::{StyleDelta, StyleKey};
use crate::invalidation::RestyleScope;
use crate::orchestration::{
    ComputedStyles, InlineStyles, PseudoStates, ShapedChMetrics, StyleCoordinator, StyleInputs,
    StyleMatchingCoordinator, TreeElement,
};

#[doc = include_str!("../../docs/api/runtime/restyle_phase.doc/RestylePhase.md")]
pub(crate) struct RestylePhase<'a> {
    /// The retained tree every match walks.
    tree: &'a RetainedTree,
    /// The live per-node interaction state `:hover` / `:focus` / `:active` match
    /// against, and the baseline half of the restyle scope.
    pseudo: &'a PseudoStates,
    /// The host-reported environment values every `env()` resolves against.
    environment: EnvironmentValues,
    /// The `ch` port: the same shaper the measure and compose passes use, so a
    /// `20ch` box and the twenty characters in it agree on one font.
    paragraph: &'a ParagraphShaper,
    /// Matches authored stylesheets (pseudo-state aware) into cascade inputs.
    matcher: &'a mut StyleMatchingCoordinator,
    /// Resolves computed styles over the retained tree.
    style: &'a mut StyleCoordinator,
    /// The cascade inputs retained across frames, rewritten in place by the
    /// incremental pass and replaced wholesale by the full one.
    inputs: &'a mut StyleInputs,
    /// The settled cascade value of every node the motion overlay has written
    /// into the computed styles — the pre-overlay stash both passes maintain.
    overlaid: &'a mut ComputedStyles,
    /// The `style` attributes, compiled and cached.
    inline: &'a mut InlineStyles,
}

impl<'a> RestylePhase<'a> {
    /// Bundles the tier one style phase drives and the state it reads.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        tree: &'a RetainedTree,
        pseudo: &'a PseudoStates,
        environment: EnvironmentValues,
        paragraph: &'a ParagraphShaper,
        matcher: &'a mut StyleMatchingCoordinator,
        style: &'a mut StyleCoordinator,
        inputs: &'a mut StyleInputs,
        overlaid: &'a mut ComputedStyles,
        inline: &'a mut InlineStyles,
    ) -> Self {
        Self {
            tree,
            pseudo,
            environment,
            paragraph,
            matcher,
            style,
            inputs,
            overlaid,
            inline,
        }
    }

    #[doc = include_str!("../../docs/api/runtime/restyle_phase.doc/scope.md")]
    pub(crate) fn scope(
        &self,
        last: Option<&StyleKey>,
        retained: &ComputedStyles,
        sheets: &[Stylesheet],
        viewport: Size,
        reconcile_roots: &[NodeId],
    ) -> Option<StyleInvalidation> {
        // Diagnostic escape hatch: `OBSCUR_FULL_RESTYLE=1` forces the pre-incremental
        // behaviour, so a suspected stale-style artifact can be attributed to this
        // pass by toggling it rather than by argument.
        if std::env::var_os("OBSCUR_FULL_RESTYLE").is_some() {
            return None;
        }
        let last = last?;
        // Stamps, not contents: both sides are clones of one compiled set, so
        // the structural comparison this replaces always ran to the end — every
        // selector of every rule — to answer "yes" on every frame that had a
        // chance of being cheap.
        if last.viewport != viewport
            || last.environment != self.environment
            || !same_sheets(&last.sheets, sheets)
        {
            return None;
        }
        // Nothing retained to reuse (no style phase has produced a map yet, or the
        // tree is empty): the full pass is the cheap one anyway.
        if retained.is_empty() {
            return None;
        }

        let mut dirty = StyleInvalidation::new();
        let pseudo_roots = RestyleScope::changed_states(&last.pseudo, self.pseudo);
        let broad = sheets.iter().any(Stylesheet::broad_child_structure);
        trace_scope(self.tree, &pseudo_roots, reconcile_roots, broad);
        for root in pseudo_roots {
            if !RestyleScope::expand(self.tree, root, &mut dirty) {
                return None;
            }
        }
        let has_sibling_rules = sheets.iter().any(Stylesheet::child_structure_sensitive);
        for &root in reconcile_roots {
            if broad {
                if !RestyleScope::expand(self.tree, root, &mut dirty) {
                    return None;
                }
                continue;
            }
            let Some(node) = self.tree.get(root) else {
                continue;
            };
            RestyleScope::mark_subtree(self.tree, root, &mut dirty);
            if has_sibling_rules && let Some(parent) = node.logical_parent().parent() {
                let children = self.tree.get(parent)?.child_nodes();
                let index = children.iter().position(|&child| child == root)?;
                for sibling in sibling_targets(self.tree, sheets, &children[index + 1..]) {
                    RestyleScope::mark_subtree(self.tree, sibling, &mut dirty);
                }
            }
        }
        Some(dirty)
    }

    #[doc = include_str!("../../docs/api/runtime/restyle_phase.doc/emit_style_value_faults.md")]
    fn emit_style_value_faults(
        &self,
        sheets: &[Stylesheet],
        environment: &MediaEnvironment,
        diagnostics: &dyn DiagnosticSink,
    ) {
        for node in
            self.matcher
                .collect_invalid_value_faults(self.tree, sheets, self.pseudo, environment)
        {
            let mut diagnostic = Diagnostic::new(
                DiagnosticCode::new("css.invalid_value"),
                DiagnosticSeverity::Warning,
                DiagnosticSubsystem::Style,
                "a declaration value could not be parsed and was dropped",
            )
            .with_recovery(RecoveryAction::Fallback);
            diagnostic.context.node = Some(node);
            diagnostics.emit(&diagnostic);
        }
    }

    #[doc = include_str!("../../docs/api/runtime/restyle_phase.doc/everything.md")]
    pub(crate) fn everything(
        &mut self,
        sheets: &[Stylesheet],
        environment: &MediaEnvironment,
        viewport: Size,
        diagnostics: &dyn DiagnosticSink,
    ) -> (ComputedStyles, usize, StyleDelta) {
        let mut inputs =
            self.matcher
                .build_inputs_with_pseudo_env(self.tree, sheets, self.pseudo, environment);
        // §4.14 origin 5, folded in after matching and before resolution: what
        // an author wrote ON the element outranks every selector that reached
        // it, and loses only to a runtime override.
        self.inline.apply_all(self.tree, &mut inputs, diagnostics);
        // Per-node style faults: a matched rule that authored an unparseable
        // declaration value (`width: not-a-length`) dropped it and kept the rest
        // (§4.13). The runtime is the only place that knows *which* element
        // matched the rule, so it raises the structured diagnostic here, on the
        // full pass that visits every node.
        self.emit_style_value_faults(sheets, environment, diagnostics);
        // The scene's paragraph orchestrator is the `ch` port, so `ch` folds to
        // pixels here with every other relative unit. It is the same shaper the
        // measure and compose passes use (disjoint fields, so a legal split
        // borrow), so `ch` and the text it sizes agree on one font.
        let ch_metrics = ShapedChMetrics::new(self.paragraph);
        let computed = self.style.resolve_with_media(
            self.tree,
            &inputs,
            viewport,
            self.environment,
            Some(&ch_metrics),
            environment,
        );
        *self.inputs = inputs;
        // Every node's style is freshly resolved, so nothing in the new map carries
        // a motion sample and the stash of pre-overlay values describes a map that
        // no longer exists.
        self.overlaid.clear();
        let resolved = computed.len();
        (computed, resolved, StyleDelta::Full)
    }

    #[doc = include_str!("../../docs/api/runtime/restyle_phase.doc/invalidated.md")]
    pub(crate) fn invalidated(
        &mut self,
        mut computed: ComputedStyles,
        sheets: &[Stylesheet],
        environment: &MediaEnvironment,
        viewport: Size,
        dirty: &StyleInvalidation,
        diagnostics: &dyn DiagnosticSink,
        reconciled: bool,
    ) -> (ComputedStyles, usize, StyleDelta) {
        // Undo the motion overlay first: the retained map is about to be treated as
        // the resolver's own output — by the incremental walk, whose non-dirty
        // entries stand in for values it did not recompute, and by the transition
        // change-detection, which must compare settled cascade targets. `replaced`
        // starts from what the overlay had put there, because that is what the
        // previous frame's LAYOUT ran on, and the layout classifier compares
        // against that.
        let mut replaced = ComputedStyles::default();
        for (node, pure) in self.overlaid.drain() {
            if let Some(slot) = computed.get_mut(&node) {
                replaced.insert(node, std::mem::replace(slot, pure));
            }
        }

        self.matcher.rematch_into(
            self.inputs,
            self.tree,
            sheets,
            self.pseudo,
            environment,
            dirty,
        );
        self.inline
            .apply_invalidated(self.tree, self.inputs, diagnostics, dirty, reconciled);
        // Removed nodes are already pruned and any new node belongs to the
        // dirty set. With a complete retained map, only the animation overlay
        // restored above can affect this frame's style delta.
        if dirty.is_empty() && computed.len() == self.tree.len() {
            return (
                computed,
                0,
                StyleDelta::Incremental {
                    replaced,
                    introduced: false,
                },
            );
        }
        // Same port as the full resolve above: an incrementally restyled node must
        // land on the identical `ch`, or a node's border would change width purely
        // because it was re-resolved by a different path.
        let ch_metrics = ShapedChMetrics::new(self.paragraph);
        let restyle = self.style.resolve_incremental_with_media(
            self.tree,
            self.inputs,
            viewport,
            self.environment,
            dirty,
            &mut computed,
            Some(&ch_metrics),
            environment,
        );
        for (node, previous) in restyle.previous {
            // The overlay snapshot above is the older of the two, so it wins.
            replaced.entry(node).or_insert(previous);
        }
        (
            computed,
            restyle.resolved,
            StyleDelta::Incremental {
                replaced,
                introduced: restyle.introduced,
            },
        )
    }

    #[doc = include_str!("../../docs/api/runtime/restyle_phase.doc/roots_of.md")]
    pub(crate) fn roots_of(
        tree: &RetainedTree,
        plan: &ReconciliationPlan,
        sheets: Option<&[Stylesheet]>,
    ) -> Option<Vec<NodeId>> {
        let mut roots = Vec::new();
        let broad =
            sheets.is_none_or(|sheets| sheets.iter().any(Stylesheet::broad_child_structure));
        let sheets = sheets.unwrap_or_default();
        let has_sibling_rules = sheets.iter().any(Stylesheet::child_structure_sensitive);
        let mut edited_parents = obscur_identity::FastSet::default();
        for op in plan.ops() {
            let parent = match *op {
                ReconciliationOp::Update { id } => {
                    roots.push(id);
                    continue;
                }
                ReconciliationOp::Insert { id, parent, .. } => {
                    if !broad {
                        roots.push(id);
                    }
                    parent
                }
                ReconciliationOp::Remove { parent, .. } => parent?,
                ReconciliationOp::Move { id, .. } => {
                    if !broad {
                        roots.push(id);
                    }
                    tree.get(id)?.logical_parent().parent()?
                }
            };
            if broad {
                roots.push(parent);
            } else if has_sibling_rules && edited_parents.insert(parent) {
                if let Some(node) = tree.get(parent) {
                    roots.extend(sibling_targets(tree, sheets, node.child_nodes()));
                }
            }
        }
        Some(roots)
    }
}

fn trace_scope(tree: &RetainedTree, pseudo: &[NodeId], reconcile: &[NodeId], broad: bool) {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if (pseudo.is_empty() && reconcile.is_empty())
        || !*ENABLED.get_or_init(|| {
            std::env::var_os("OBSCUR_RESTYLE_TRACE").is_some_and(|value| value == "1")
        })
    {
        return;
    }
    let describe = |roots: &[NodeId]| {
        roots
            .iter()
            .filter_map(|id| {
                let attributes = tree.get(*id)?.attributes();
                Some(format!(
                    "{id:?}:{}:{:?}",
                    attributes.local_name().unwrap_or("-"),
                    attributes.classes()
                ))
            })
            .collect::<Vec<_>>()
    };
    println!(
        "[restyle-roots] broad={broad} pseudo={:?} reconcile={:?}",
        describe(pseudo),
        describe(reconcile)
    );
}

fn sibling_targets<'a>(
    tree: &'a RetainedTree,
    sheets: &'a [Stylesheet],
    children: &'a [NodeId],
) -> impl Iterator<Item = NodeId> + 'a {
    children.iter().copied().filter(move |&child| {
        let element = TreeElement::new(tree, child);
        sheets.iter().any(|sheet| sheet.is_sibling_target(&element))
    })
}

#[doc = include_str!("../../docs/api/runtime/restyle_phase.doc/same_sheets.md")]
fn same_sheets(last: &[Stylesheet], now: &[Stylesheet]) -> bool {
    last.len() == now.len()
        && last
            .iter()
            .zip(now)
            .all(|(before, after)| before.stamp() == after.stamp())
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_style::cascade::CascadeOrigin;

    #[test]
    fn the_same_compiled_sheets_are_recognised_through_any_number_of_clones() {
        // The frame path is clone-heavy on purpose (`author_for_frame` makes two,
        // `cascade` a third), and every one of those must still read as "the
        // sheets have not changed" or no frame is ever incremental.
        let installed = vec![
            Stylesheet::new(CascadeOrigin::SystemDefault, Vec::new()),
            Stylesheet::new(CascadeOrigin::GlobalApp, Vec::new()),
        ];
        let remembered = installed.clone();
        assert!(same_sheets(&remembered, &installed.clone().clone()));
    }

    #[test]
    fn sheets_compiled_afresh_are_not_taken_for_the_ones_they_replaced() {
        // Two sheets with identical (empty) rule lists: content comparison calls
        // these the same, and a skin change that recompiled to the same bytes
        // would then keep the previous frame's styles. The stamp says no.
        let before = vec![Stylesheet::new(CascadeOrigin::GlobalApp, Vec::new())];
        let after = vec![Stylesheet::new(CascadeOrigin::GlobalApp, Vec::new())];
        assert_eq!(before, after, "content-wise they are equal");
        assert!(!same_sheets(&before, &after), "identity-wise they are not");
    }

    #[test]
    fn a_set_that_gained_or_lost_a_sheet_is_never_the_same_set() {
        let one = vec![Stylesheet::new(CascadeOrigin::GlobalApp, Vec::new())];
        let two = vec![
            one[0].clone(),
            Stylesheet::new(CascadeOrigin::Theme, Vec::new()),
        ];
        assert!(!same_sheets(&one, &two));
        assert!(!same_sheets(&two, &one));
        assert!(same_sheets(&[], &[]));
    }

    #[test]
    fn a_plan_that_changed_nothing_bounds_the_restyle_to_no_roots_at_all() {
        let tree = RetainedTree::new();
        let plan = ReconciliationPlan::default();
        assert_eq!(RestylePhase::roots_of(&tree, &plan, None), Some(Vec::new()));
    }
}
