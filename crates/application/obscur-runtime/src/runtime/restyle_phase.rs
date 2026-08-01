#![doc = include_str!("restyle_phase.doc/_module.md")]

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
    ComputedStyles, PseudoStates, ShapedChMetrics, StyleCoordinator, StyleInputs,
    StyleMatchingCoordinator,
};

#[doc = include_str!("restyle_phase.doc/RestylePhase.md")]
pub(crate) struct RestylePhase<'a> {
    #[doc = include_str!("restyle_phase.doc/tree.md")]
    tree: &'a RetainedTree,
    #[doc = include_str!("restyle_phase.doc/pseudo.md")]
    pseudo: &'a PseudoStates,
    #[doc = include_str!("restyle_phase.doc/environment.md")]
    environment: EnvironmentValues,
    #[doc = include_str!("restyle_phase.doc/paragraph.md")]
    paragraph: &'a ParagraphShaper,
    #[doc = include_str!("restyle_phase.doc/matcher.md")]
    matcher: &'a mut StyleMatchingCoordinator,
    #[doc = include_str!("restyle_phase.doc/style.md")]
    style: &'a mut StyleCoordinator,
    #[doc = include_str!("restyle_phase.doc/inputs.md")]
    inputs: &'a mut StyleInputs,
    #[doc = include_str!("restyle_phase.doc/overlaid.md")]
    overlaid: &'a mut ComputedStyles,
}

impl<'a> RestylePhase<'a> {
    #[doc = include_str!("restyle_phase.doc/new.md")]
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
        }
    }

    #[doc = include_str!("restyle_phase.doc/scope.md")]
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
        if last.viewport != viewport
            || last.environment != self.environment
            || last.sheets != sheets
        {
            return None;
        }
        // Nothing retained to reuse (no style phase has produced a map yet, or the
        // tree is empty): the full pass is the cheap one anyway.
        if retained.is_empty() {
            return None;
        }

        let mut dirty = StyleInvalidation::new();
        for root in RestyleScope::changed_states(&last.pseudo, self.pseudo) {
            if !RestyleScope::expand(self.tree, root, &mut dirty) {
                // The scope could not be bounded; a partial set would be a wrong
                // one, so widen to everything.
                return None;
            }
        }
        // The nodes this frame's tree edit touched, expanded the same way: a
        // reconcile is no longer an automatic full pass.
        for &root in reconcile_roots {
            if !RestyleScope::expand(self.tree, root, &mut dirty) {
                return None;
            }
        }
        Some(dirty)
    }

    #[doc = include_str!("restyle_phase.doc/emit_style_value_faults.md")]
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

    #[doc = include_str!("restyle_phase.doc/everything.md")]
    pub(crate) fn everything(
        &mut self,
        sheets: &[Stylesheet],
        environment: &MediaEnvironment,
        viewport: Size,
        diagnostics: &dyn DiagnosticSink,
    ) -> (ComputedStyles, usize, StyleDelta) {
        let inputs =
            self.matcher
                .build_inputs_with_pseudo_env(self.tree, sheets, self.pseudo, environment);
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
        let computed = self.style.resolve(
            self.tree,
            &inputs,
            viewport,
            self.environment,
            Some(&ch_metrics),
        );
        *self.inputs = inputs;
        // Every node's style is freshly resolved, so nothing in the new map carries
        // a motion sample and the stash of pre-overlay values describes a map that
        // no longer exists.
        self.overlaid.clear();
        let resolved = computed.len();
        (computed, resolved, StyleDelta::Full)
    }

    #[doc = include_str!("restyle_phase.doc/invalidated.md")]
    pub(crate) fn invalidated(
        &mut self,
        mut computed: ComputedStyles,
        sheets: &[Stylesheet],
        environment: &MediaEnvironment,
        viewport: Size,
        dirty: &StyleInvalidation,
    ) -> (ComputedStyles, usize, StyleDelta) {
        // Undo the motion overlay first: the retained map is about to be treated as
        // the resolver's own output — by the incremental walk, whose non-dirty
        // entries stand in for values it did not recompute, and by the transition
        // change-detection, which must compare settled cascade targets. `replaced`
        // starts from what the overlay had put there, because that is what the
        // previous frame's LAYOUT ran on, and the layout classifier compares
        // against that.
        let mut replaced = ComputedStyles::new();
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
        // Same port as the full resolve above: an incrementally restyled node must
        // land on the identical `ch`, or a node's border would change width purely
        // because it was re-resolved by a different path.
        let ch_metrics = ShapedChMetrics::new(self.paragraph);
        let restyle = self.style.resolve_incremental(
            self.tree,
            self.inputs,
            viewport,
            self.environment,
            dirty,
            &mut computed,
            Some(&ch_metrics),
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

    #[doc = include_str!("restyle_phase.doc/roots_of.md")]
    pub(crate) fn roots_of(tree: &RetainedTree, plan: &ReconciliationPlan) -> Option<Vec<NodeId>> {
        let mut roots = Vec::new();
        for op in plan.ops() {
            match op {
                // In place: the node's own subtree and what follows it.
                ReconciliationOp::Update { id } => roots.push(*id),
                // Structural: the parent, so both directions of sibling counting and
                // the parent's own `:empty` are covered.
                ReconciliationOp::Insert { parent, .. } => roots.push(*parent),
                ReconciliationOp::Remove { parent, .. } => roots.push((*parent)?),
                ReconciliationOp::Move { id, .. } => {
                    roots.push(tree.get(*id)?.logical_parent().parent()?);
                }
            }
        }
        Some(roots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plan_that_changed_nothing_bounds_the_restyle_to_no_roots_at_all() {
        let tree = RetainedTree::new();
        let plan = ReconciliationPlan::default();
        assert_eq!(RestylePhase::roots_of(&tree, &plan), Some(Vec::new()));
    }
}
