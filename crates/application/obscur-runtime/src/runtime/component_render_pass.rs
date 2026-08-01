#![doc = include_str!("component_render_pass.doc/_module.md")]

use std::collections::HashSet;

use obscur_diagnostics::{Diagnostic, DiagnosticSink, DiagnosticSubsystem, RecoveryAction};
use obscur_identity::NodeId;
use obscur_tree::reconciliation::Reconciler;
use obscur_tree::tree::RetainedTree;
use obscur_view::boundary::ComponentFailure;
use obscur_view::view::{View, project_slots};

use super::FrameTelemetry;
use crate::component::{ComponentStore, GuardedRender};
use crate::recovery::RecoveryResult;
use crate::transaction::ReconcileEnactor;

#[doc = include_str!("component_render_pass.doc/MAX_PASSES.md")]
const MAX_PASSES: usize = 32;

#[doc = include_str!("component_render_pass.doc/ComponentRenderPass.md")]
pub(crate) struct ComponentRenderPass<'a> {
    #[doc = include_str!("component_render_pass.doc/tree.md")]
    tree: &'a mut RetainedTree,
    #[doc = include_str!("component_render_pass.doc/components.md")]
    components: &'a mut ComponentStore,
    #[doc = include_str!("component_render_pass.doc/reconciler.md")]
    reconciler: &'a Reconciler,
    #[doc = include_str!("component_render_pass.doc/enactor.md")]
    enactor: &'a mut ReconcileEnactor,
    #[doc = include_str!("component_render_pass.doc/telemetry.md")]
    telemetry: &'a mut FrameTelemetry,
    #[doc = include_str!("component_render_pass.doc/diagnostics.md")]
    diagnostics: &'a dyn DiagnosticSink,
}

impl<'a> ComponentRenderPass<'a> {
    #[doc = include_str!("component_render_pass.doc/new.md")]
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

    #[doc = include_str!("component_render_pass.doc/run.md")]
    pub(crate) fn run(&mut self) -> bool {
        // The common case by far: no host registrations, nothing mounted.
        if self.components.is_empty() {
            return false;
        }
        let mut pending: Vec<NodeId> = self.components.stale_nodes();
        let mut done: HashSet<NodeId> = HashSet::new();
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
                    Some(retained) => project_slots(view, retained.slot_content()),
                    None => view,
                };

                let (_commit, plan) = self
                    .enactor
                    .commit_subtree(self.reconciler, self.tree, host, view.root())
                    .expect("a component rerender is a single, structurally-valid Rerender");
                changed |= !plan.is_empty();
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
        changed
    }

    #[doc = include_str!("component_render_pass.doc/report_failure.md")]
    fn report_failure(&self, node: NodeId, diagnostic: &Diagnostic, recovery: RecoveryAction) {
        let mut diagnostic = diagnostic.clone();
        diagnostic.context.node = Some(node);
        diagnostic.recovery = recovery;
        self.diagnostics.emit(&diagnostic);
    }

    #[doc = include_str!("component_render_pass.doc/contain_escaped.md")]
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

    #[doc = include_str!("component_render_pass.doc/enclosing_boundary.md")]
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
        assert!(!pass.run());
    }
}
