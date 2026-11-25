//! The frame's component-render phase: [`ComponentRenderPass`].

use std::collections::HashSet;

use obscur_diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSink, DiagnosticSubsystem,
    RecoveryAction,
};
use obscur_identity::{FastSet, NodeId};
use obscur_style::matching::Stylesheet;
use obscur_tree::node::{NodeKind, RetainedNode};
use obscur_tree::reconciliation::Reconciler;
use obscur_tree::tree::RetainedTree;
use obscur_view::boundary::ComponentFailure;
use obscur_view::view::{SlotMismatch, View, project_slots};

use super::{FrameTelemetry, RestylePhase};
use crate::component::{ComponentStore, GuardedRender};
use crate::recovery::RecoveryResult;
use crate::transaction::ReconcileEnactor;

use super::reconcile_counts::ReconcileCounts;

#[doc = include_str!("../../docs/api/runtime/component_render_pass.doc/MAX_PASSES.md")]
const MAX_PASSES: usize = 32;

#[doc = include_str!("../../docs/api/runtime/component_render_pass.doc/ComponentRenderPass.md")]
pub(crate) struct ComponentRenderPass<'a> {
    /// The retained tree the sub-reconciles commit into.
    tree: &'a mut RetainedTree,
    /// The mounted instances, their staleness and their boundary policies.
    components: &'a mut ComponentStore,
    /// The reconciler each component's output is diffed with.
    reconciler: &'a Reconciler,
    /// The transaction boundary every sub-reconcile is enacted through, so a
    /// component's output is committed and versioned exactly like the root's.
    enactor: &'a mut ReconcileEnactor,
    /// Where the `component.rendered` / `component.props_errors` counters land.
    telemetry: &'a mut FrameTelemetry,
    /// Where a failed render and the recovery taken for it are reported (§1.6).
    diagnostics: &'a dyn DiagnosticSink,
}

impl<'a> ComponentRenderPass<'a> {
    /// Bundles the state one component-render phase reads and writes.
    pub(crate) fn new(
        tree: &'a mut RetainedTree,
        components: &'a mut ComponentStore,
        reconciler: &'a Reconciler,
        enactor: &'a mut ReconcileEnactor,
        telemetry: &'a mut FrameTelemetry,
        diagnostics: &'a dyn DiagnosticSink,
    ) -> Self {
        Self {
            tree,
            components,
            reconciler,
            enactor,
            telemetry,
            diagnostics,
        }
    }

    #[doc = include_str!("../../docs/api/runtime/component_render_pass.doc/run.md")]
    pub(crate) fn run(
        &mut self,
        previous_sheets: Option<&[Stylesheet]>,
        mut counts: Option<&mut ReconcileCounts>,
    ) -> (bool, Option<Vec<NodeId>>) {
        // The common case by far: no host registrations, nothing mounted.
        if self.components.is_empty() {
            return (false, Some(Vec::new()));
        }
        // The restyle roots every sub-reconcile below implies. This used to be
        // thrown away and answered as "unknowable", which made the style phase
        // restyle the whole tree on any frame where a single component
        // re-rendered — and in an application built out of components, that is
        // every interactive frame. The plans are right here; they are knowable.
        let mut roots: Option<Vec<NodeId>> = Some(Vec::new());
        let mut pending: Vec<NodeId> = self.components.stale_nodes();
        let mut done: FastSet<NodeId> = HashSet::default();
        let mut changed = false;
        let mut errors: u64 = 0;
        let mut rendered: u64 = 0;

        for _ in 0..MAX_PASSES {
            if pending.is_empty() {
                break;
            }
            let mut next = Vec::new();
            for node in std::mem::take(&mut pending) {
                // At most once per frame, however many times it is re-requested.
                if !done.insert(node) {
                    continue;
                }
                // The node may have been removed by an earlier pass in this loop.
                let Some(props) = self.tree.get(node).map(|n| n.props().to_vec()) else {
                    continue;
                };
                // The generations book is read-only during a render and lives on
                // the store alongside the instance, so take a snapshot of it to
                // keep the two borrows disjoint.
                let signals = self.components.signals().clone();
                // Scoped so the store's borrow ends before the boundary's answer
                // is enacted, which touches the tree and other instances.
                let outcome = {
                    let Some(mounted) = self.components.get_mut(node) else {
                        continue;
                    };
                    mounted.render(&props, &signals)
                };
                let committed = match outcome {
                    GuardedRender::Rendered(view) => {
                        rendered += 1;
                        Some((node, view))
                    }
                    GuardedRender::Failed {
                        failure,
                        result,
                        fallback,
                    } => {
                        errors += 1;
                        self.report_failure(node, failure.diagnostic(), result.recovery_action());
                        match fallback {
                            // The boundary is out of retries: this component's
                            // own subtree becomes its fallback UI.
                            Some(view) => Some((node, view)),
                            None if result.is_propagated() => self.contain_escaped(node, &failure),
                            // A retry: nothing is committed, the component stays
                            // stale, and the frame end asks for the frame that
                            // spends the next of its budget.
                            None => None,
                        }
                    }
                };
                let Some((host, view)) = committed else {
                    continue;
                };

                // §2.12: the component said *where* its authored children go by
                // rendering a `ViewNode::slot("…")` outlet; the reconciler kept
                // *what* goes there on the host node. Join them before the output
                // becomes tree, so an outlet is substituted away rather than
                // retained, and the projected content reconciles as ordinary
                // structure (and mounts, if it names a component of its own).
                //
                // Free for the components that declare no outlet, which is all of
                // them until one takes children: `project_slots` scans for an
                // outlet and returns the view untouched when there is none.
                let view = match self.tree.get(host) {
                    Some(retained) => {
                        // Before the outlets are substituted away, while both
                        // halves still exist: a name used on either side that
                        // the contract declares on neither is dropped in
                        // silence, and this is what says so.
                        self.report_slot_mismatches(host, &view, retained);
                        project_slots(view, retained.slot_content())
                    }
                    None => view,
                };

                let (_commit, plan) = self
                    .enactor
                    .commit_subtree(self.reconciler, self.tree, host, view.root())
                    .expect("a component rerender is a single, structurally-valid Rerender");
                changed |= !plan.is_empty();
                if let Some(counts) = counts.as_deref_mut() {
                    counts.record(&plan);
                }
                // A component whose output matched what was already retained
                // edited nothing, and so dirties nothing — that is the whole
                // saving, and it is the common case in a tree of components
                // where one of them was pressed.
                if !plan.is_empty()
                    && let Some(collected) = roots.as_mut()
                {
                    match RestylePhase::roots_of(self.tree, &plan, previous_sheets) {
                        Some(from_plan) => collected.extend(from_plan),
                        // A sub-plan that cannot be bounded widens the whole
                        // frame, exactly as an unboundable root reconcile does.
                        None => roots = None,
                    }
                }
                self.components.sync(self.tree, &plan);
                // Only newly inserted nodes can have minted new instances.
                let fresh: Vec<NodeId> = plan
                    .iter()
                    .filter(|op| op.is_insert())
                    .map(|op| op.id())
                    .filter(|id| self.components.get(*id).is_some())
                    .collect();
                next.extend(fresh);
            }
            pending = next;
        }

        self.telemetry
            .count(DiagnosticSubsystem::Tree, "component.rendered", rendered);
        if errors > 0 {
            self.telemetry
                .count(DiagnosticSubsystem::Tree, "component.props_errors", errors);
        }
        (changed, roots)
    }

    #[doc = include_str!("../../docs/api/runtime/component_render_pass.doc/report_slot_mismatches.md")]
    fn report_slot_mismatches(&self, node: NodeId, output: &View, retained: &RetainedNode) {
        let NodeKind::Element { component } = retained.kind else {
            return;
        };
        let Some(contract) = self.components.contract(component) else {
            return;
        };
        for mismatch in SlotMismatch::find(contract, output, retained.slot_content()) {
            let message = match &mismatch {
                SlotMismatch::UndeclaredOutlet(name) if name.is_empty() => {
                    "this component renders the default `<slot/>` outlet — the one \
                     that takes every child naming no slot — but its contract \
                     declares no default slot (spec §2.12, §2.13). Declare it with \
                     `#[contract(slot(default))]`, or give the outlet the name of a \
                     slot the contract does state"
                        .to_owned()
                }
                SlotMismatch::UndeclaredOutlet(name) => format!(
                    "this component renders a `<slot name=\"{name}\"/>` outlet, and \
                     its contract declares no slot called `{name}` (spec §2.12, \
                     §2.13). Nothing can ever be assigned to that name, so the \
                     outlet renders its fallback if it has one and vanishes if it \
                     does not — check the spelling against the contract"
                ),
                SlotMismatch::UndeclaredAssignment(name) if name.is_empty() => {
                    "an authored child names no slot, and this component declares no \
                     default slot to take it (spec §2.12), so it is dropped rather \
                     than rendered. Assign the child to a slot the contract states \
                     with `slot=\"…\"`, or declare the default slot on the component"
                        .to_owned()
                }
                SlotMismatch::UndeclaredAssignment(name) => format!(
                    "an authored child asks for the slot `{name}`, which this \
                     component's contract does not declare (spec §2.12), so it is \
                     dropped rather than rendered — check the spelling against the \
                     component's declared slots"
                ),
            };
            let mut diagnostic = Diagnostic::new(
                DiagnosticCode::new("view.undeclared_slot"),
                DiagnosticSeverity::Error,
                contract.subsystem,
                message,
            );
            diagnostic.context.node = Some(node);
            self.diagnostics.emit(&diagnostic);
        }
    }

    #[doc = include_str!("../../docs/api/runtime/component_render_pass.doc/report_failure.md")]
    fn report_failure(&self, node: NodeId, diagnostic: &Diagnostic, recovery: RecoveryAction) {
        let mut diagnostic = diagnostic.clone();
        diagnostic.context.node = Some(node);
        diagnostic.recovery = recovery;
        self.diagnostics.emit(&diagnostic);
    }

    #[doc = include_str!("../../docs/api/runtime/component_render_pass.doc/contain_escaped.md")]
    fn contain_escaped(
        &mut self,
        node: NodeId,
        failure: &ComponentFailure,
    ) -> Option<(NodeId, View)> {
        let host = self.enclosing_boundary(node)?;
        let mut escaped = failure.diagnostic().clone();
        escaped.message = format!(
            "{} (escaped the disabled boundary of a descendant component)",
            escaped.message
        );
        let (contained, fallback) = self.components.get_mut(host)?.contain_escaped(escaped);
        self.report_failure(
            host,
            contained.diagnostic(),
            RecoveryResult::FellBack.recovery_action(),
        );
        Some((host, fallback))
    }

    #[doc = include_str!("../../docs/api/runtime/component_render_pass.doc/enclosing_boundary.md")]
    fn enclosing_boundary(&self, node: NodeId) -> Option<NodeId> {
        let mut current = self.tree.get(node)?.logical_parent.parent();
        while let Some(id) = current {
            if let Some(mounted) = self.components.get(id)
                && mounted.boundary_policy().is_enabled()
            {
                return Some(id);
            }
            current = self.tree.get(id)?.logical_parent.parent();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use obscur_diagnostics::NullDiagnosticSink;

    use super::*;

    #[test]
    fn a_runtime_with_no_registered_components_renders_nothing_and_changes_nothing() {
        let mut tree = RetainedTree::new();
        let mut components = ComponentStore::new();
        let reconciler = Reconciler::new();
        let mut enactor = ReconcileEnactor::new();
        let mut telemetry = FrameTelemetry::new();
        let sink = NullDiagnosticSink;
        let mut pass = ComponentRenderPass::new(
            &mut tree,
            &mut components,
            &reconciler,
            &mut enactor,
            &mut telemetry,
            &sink,
        );
        assert_eq!(
            pass.run(None, None),
            (false, Some(Vec::new())),
            "nothing mounted names no restyle root — not `None`, which would \
             widen every frame of a component-free application to a full restyle"
        );
    }
}
