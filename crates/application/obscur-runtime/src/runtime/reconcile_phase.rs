#![doc = include_str!("reconcile_phase.doc/_module.md")]

use obscur_diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSink, DiagnosticSubsystem,
    RecoveryAction,
};
use obscur_identity::NodeId;
use obscur_tree::reconciliation::{Reconciler, ReconciliationPlan};
use obscur_tree::tree::RetainedTree;
use obscur_view::view::{View, has_outlet, has_virtual_collection};

use super::{
    ComponentRenderPass, DesiredView, FramePhase, FrameTelemetry, RestylePhase, ScrollOffsets,
    VirtualLists,
};
use crate::component::ComponentStore;
use crate::frame::FrameContext;
use crate::transaction::ReconcileEnactor;

#[doc = include_str!("reconcile_phase.doc/ReconcilePhase.md")]
pub(crate) struct ReconcilePhase<'a> {
    #[doc = include_str!("reconcile_phase.doc/tree.md")]
    tree: &'a mut RetainedTree,
    #[doc = include_str!("reconcile_phase.doc/reconciler.md")]
    reconciler: &'a Reconciler,
    #[doc = include_str!("reconcile_phase.doc/enactor.md")]
    enactor: &'a mut ReconcileEnactor,
    #[doc = include_str!("reconcile_phase.doc/components.md")]
    components: &'a mut ComponentStore,
    #[doc = include_str!("reconcile_phase.doc/virtual_lists.md")]
    virtual_lists: &'a mut VirtualLists,
    #[doc = include_str!("reconcile_phase.doc/scroll.md")]
    scroll: &'a mut ScrollOffsets,
    #[doc = include_str!("reconcile_phase.doc/telemetry.md")]
    telemetry: &'a mut FrameTelemetry,
    #[doc = include_str!("reconcile_phase.doc/diagnostics.md")]
    diagnostics: &'a dyn DiagnosticSink,
}

impl<'a> ReconcilePhase<'a> {
    #[doc = include_str!("reconcile_phase.doc/new.md")]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        tree: &'a mut RetainedTree,
        reconciler: &'a Reconciler,
        enactor: &'a mut ReconcileEnactor,
        components: &'a mut ComponentStore,
        virtual_lists: &'a mut VirtualLists,
        scroll: &'a mut ScrollOffsets,
        telemetry: &'a mut FrameTelemetry,
        diagnostics: &'a dyn DiagnosticSink,
    ) -> Self {
        Self {
            tree,
            reconciler,
            enactor,
            components,
            virtual_lists,
            scroll,
            telemetry,
            diagnostics,
        }
    }

    #[doc = include_str!("reconcile_phase.doc/run.md")]
    pub(crate) fn run(
        &mut self,
        ctx: &FrameContext,
        desired: DesiredView<'_>,
        viewport_scroll_y: f32,
    ) -> (bool, Option<Vec<NodeId>>) {
        // Materialize the desired tree HERE — after phase 3's input dispatch,
        // not before it. A reactive frame's builder reads current state, and a
        // dispatched input handler may have just committed to that state, so
        // building any earlier would reconcile the pre-dispatch view and paint
        // the handler's mutation a frame late (§3.4: dispatch precedes commit).
        let built_view;
        let resolved: &View = match desired {
            DesiredView::Ready(view) => view,
            DesiredView::Build(build) => {
                built_view = build();
                &built_view
            }
        };
        // §4.9: realize every `<VirtualFor>` HERE — the one place that owns the
        // viewport. Each marker (spec §2.11) is replaced by a window of its
        // template whose length comes from `VirtualLayout` against this frame's
        // viewport, so the tree that reconciles below holds tens of rows, not
        // one per item. A view with no marker is returned untouched.
        let realized_view;
        let desired: &View = if has_virtual_collection(resolved.root().nodes()) {
            // The scroll of a list whose content the app replaced is reset in the
            // same call that computes the window from it, so the two cannot come
            // apart — see [`VirtualLists::realize`].
            realized_view = self.virtual_lists.realize(
                resolved,
                ctx.viewport().height,
                viewport_scroll_y,
                self.scroll,
            );
            &realized_view
        } else {
            resolved
        };
        // §2.12: `desired` is the application's own view, so it is nobody's
        // render output and no `project_slots` will ever run over it. An
        // outlet in it — at the root or anywhere in a component host's
        // authored children, which are cloned into a component's output
        // un-projected — is a stray. The reconcile below refuses to
        // materialize it; this is what says so out loud.
        self.report_stray_outlets(desired);
        let (commit, reconcile_plan) = self
            .enactor
            .commit_root(self.reconciler, self.tree, desired)
            .expect("a root reconcile is a single, structurally-valid ReplaceRoot");
        let changed = !commit.is_empty();
        // The restyle roots this reconcile implies, so the style phase can
        // restyle what the tree edit touched instead of everything. `None`
        // means the edit could not be bounded and the full pass is the only
        // correct answer.
        self.emit_duplicate_key_faults(&reconcile_plan);
        self.virtual_lists.track_containers(self.tree);
        let mut roots = RestylePhase::roots_of(self.tree, &reconcile_plan);
        // 1b. Mount an instance for every inserted component-backed node and
        //     unmount every removed one, from the plan's structural deltas.
        //     A host that registered no component kinds pays nothing here.
        self.components.sync(self.tree, &reconcile_plan);
        // 1c. Render every mounted component and reconcile its output into
        //     its own children, so a component-backed node's subtree exists
        //     before style/layout read the tree.
        let rendered = ComponentRenderPass::new(
            self.tree,
            self.components,
            self.reconciler,
            self.enactor,
            self.telemetry,
            self.diagnostics,
        )
        .run();
        // A component sub-reconcile edits the tree through its own plan, which
        // this frame never sees, so its roots are unknowable here: widen.
        if rendered {
            roots = None;
        }
        let changed = changed || rendered;
        // Telemetry: mark the reconcile phase and count the resulting nodes,
        // budgeting the retained-tree size (spec §7.9).
        let nodes = self.tree.len() as u64;
        self.telemetry
            .record_phase(FramePhase::Reconcile, nodes, ctx.id());
        (changed, roots)
    }

    #[doc = include_str!("reconcile_phase.doc/report_stray_outlets.md")]
    fn report_stray_outlets(&self, desired: &View) {
        if !has_outlet(desired.root().nodes()) {
            return;
        }
        self.diagnostics.emit(&Diagnostic::new(
            DiagnosticCode::new("view.stray_slot_outlet"),
            DiagnosticSeverity::Error,
            DiagnosticSubsystem::Tree,
            "a `<slot>` outlet was authored outside a component's render output \
             (spec §2.12): an outlet marks where a component's authored children \
             are projected, and only a component's own render is projected, so \
             this one names a slot nothing will ever fill. It is dropped rather \
             than rendered as an empty box. Author the outlet in the `render` of \
             the component that declares the slot",
        ));
    }

    #[doc = include_str!("reconcile_phase.doc/emit_duplicate_key_faults.md")]
    fn emit_duplicate_key_faults(&self, plan: &ReconciliationPlan) {
        for duplicate in plan.duplicate_keys() {
            let mut diagnostic = Diagnostic::new(
                DiagnosticCode::new("tree.duplicate_key"),
                DiagnosticSeverity::Warning,
                DiagnosticSubsystem::Tree,
                "several sibling children claim one key, so the group is rebuilt \
                 every frame instead of being reused",
            )
            .with_recovery(RecoveryAction::Fallback);
            diagnostic.context.node = Some(duplicate.parent);
            self.diagnostics.emit(&diagnostic);
        }
    }
}
