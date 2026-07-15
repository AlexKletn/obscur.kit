//! [`ReconcileEnactor`]: enacts a frame's root reconcile through the atomic
//! transaction commit boundary (spec §3.2, §3.4 phase 4).

use obscur_identity::{NodeId, VersionStamp};
use obscur_tree::reconciliation::{Reconciler, ReconciliationPlan};
use obscur_tree::tree::RetainedTree;
use obscur_view::view::{View, ViewFragment};

use super::transaction_commit::TransactionCommit;
use super::transaction_error::TransactionError;
use super::transaction_validator::TransactionValidator;
use super::ui_operation::UiOperation;
use super::ui_transaction::UiTransaction;

/// Enacts a frame's tree mutation **through** the UI-transaction boundary so the
/// spec §3.2 rule — "mutations are normalized into validated `UiOps` and
/// committed atomically between frames" — is realized on the live frame path
/// rather than only unit-tested.
///
/// The frame pipeline used to mutate the retained tree by calling
/// [`Reconciler::reconcile_root`] directly, bypassing the transaction layer. This
/// enactor makes the reconcile step go through the validated commit boundary
/// instead, while keeping reconciliation itself as the underlying mechanism:
///
/// 1. **Stage** the intent as a [`UiTransaction`] carrying a single
///    [`UiOperation::ReplaceRoot`] — the normalized "the root desired output
///    changed, reconcile the whole tree" operation.
/// 2. **Validate** the staged batch with a [`TransactionValidator`] *before*
///    application; a rejected batch aborts with no mutation (a single
///    `ReplaceRoot` is always structurally valid, so this never rejects in
///    practice, but the gate is real and observable).
/// 3. **Apply** the reconcile as the commit's effect: `reconcile_root` performs
///    the actual retained-tree mutation.
/// 4. **Commit** atomically: when the reconcile changed the tree, advance the
///    [`VersionStamp`] and record a [`TransactionCommit`] carrying the applied
///    operation stamped with the new version; an idempotent (no-op) reconcile
///    produces an empty commit and leaves the version untouched.
///
/// Because the effect is exactly the previous `reconcile_root` call, the retained
/// tree — and therefore every downstream style/layout/scene result — is
/// **identical** to the pre-boundary behavior. What changes is only that the
/// mutation now flows through a validated, versioned commit that a caller can
/// observe (via [`version`](Self::version) and the returned [`TransactionCommit`]).
#[derive(Debug, Clone, Default)]
pub struct ReconcileEnactor {
    /// The stateless structural validator each staged batch passes through.
    validator: TransactionValidator,
    /// The last version stamped at a committing frame boundary. Advances only
    /// when a commit actually changed the tree.
    version: VersionStamp,
}

impl ReconcileEnactor {
    /// Creates an enactor whose commit version starts at the default stamp.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            validator: TransactionValidator::new(),
            version: VersionStamp::from_raw(0),
        }
    }

    /// Returns the version stamped at the most recent committing frame boundary.
    ///
    /// It advances by one every time [`commit_root`](Self::commit_root) applies a
    /// reconcile that actually changed the tree, and is unchanged by a no-op
    /// (idempotent) reconcile, so a caller can observe whether a frame committed a
    /// real tree change.
    #[must_use]
    pub const fn version(&self) -> VersionStamp {
        self.version
    }

    /// The validator this enactor gates its own commits with.
    ///
    /// Exposed so another commit path — a
    /// [`ProvisionalTransaction`](super::ProvisionalTransaction) being committed,
    /// say — is checked against **this** rulebook rather than a second one it
    /// constructed for itself. The validator is stateless, so sharing it costs
    /// nothing; the point is that there is one set of rules, and no way for a
    /// caller to drift from it.
    #[must_use]
    pub const fn validator(&self) -> &TransactionValidator {
        &self.validator
    }

    /// Enacts a root reconcile of `tree` toward `desired` through the atomic
    /// transaction commit boundary and returns the committed record.
    ///
    /// The returned [`TransactionCommit`] is non-empty (and carries the advanced
    /// [`version`](Self::version)) exactly when the reconcile changed the tree;
    /// it is empty when the reconcile was an idempotent no-op. Callers derive the
    /// frame's `reconciled` flag from [`TransactionCommit::is_empty`], matching
    /// the previous `!plan.is_empty()` semantics precisely.
    ///
    /// The underlying [`ReconciliationPlan`] is returned alongside it, because the
    /// commit records the *transaction* (one normalized `ReplaceRoot`) while the
    /// plan records the *structural deltas* — which nodes were inserted, updated,
    /// moved or removed. A caller that maintains per-node state (component
    /// instances) needs those deltas; a tree walk would have to rediscover what
    /// the plan already states.
    ///
    /// # Errors
    ///
    /// Returns the [`TransactionError`] from validation if the staged batch is
    /// rejected *before* any mutation. The batch is a single
    /// [`UiOperation::ReplaceRoot`], which is always structurally valid, so this
    /// never returns `Err` in practice — the `Result` keeps the validation gate
    /// honest and total rather than assumed.
    pub fn commit_root(
        &mut self,
        reconciler: &Reconciler,
        tree: &mut RetainedTree,
        desired: &View,
    ) -> Result<(TransactionCommit, ReconciliationPlan), TransactionError> {
        // 1. Stage the normalized intent: a whole-tree root reconcile.
        let mut staged = UiTransaction::new();
        staged.push(UiOperation::ReplaceRoot);

        // 2. Validate before application. On rejection nothing is mutated.
        self.validator.validate(&staged)?;

        // 3. Apply: reconciliation is the underlying mechanism that performs the
        //    actual retained-tree mutation, exactly as before.
        let plan = reconciler.reconcile_root(tree, desired);

        // 4. Commit atomically. Only a reconcile that changed the tree advances
        //    the version and carries operations; an idempotent pass commits
        //    empty and leaves the stamp untouched.
        if plan.is_empty() {
            return Ok((TransactionCommit::new(Vec::new(), self.version), plan));
        }
        self.version = self.version.bump();
        Ok((
            TransactionCommit::new(staged.into_operations(), self.version),
            plan,
        ))
    }

    /// Enacts a **subtree** reconcile — one mounted component's freshly rendered
    /// output into its own children — through the same commit boundary, and
    /// returns the committed record plus the structural deltas.
    ///
    /// This is the [`UiOperation::Rerender`] counterpart of
    /// [`commit_root`](Self::commit_root): where a root replacement restates the
    /// whole desired tree, a rerender restates exactly one component's output.
    /// The tree below `node` is the only thing that can change, so a component
    /// whose local state advanced costs its own subtree rather than the document.
    ///
    /// Version semantics match `commit_root` exactly: an idempotent rerender —
    /// a component that re-rendered to the same output — commits empty and leaves
    /// the stamp untouched, so a caller can tell a real change from a no-op.
    ///
    /// # Errors
    ///
    /// Returns the [`TransactionError`] from validation if the staged batch is
    /// rejected *before* any mutation. A single [`UiOperation::Rerender`] is
    /// always structurally valid, so this does not return `Err` in practice; the
    /// `Result` keeps the validation gate total rather than assumed.
    pub fn commit_subtree(
        &mut self,
        reconciler: &Reconciler,
        tree: &mut RetainedTree,
        node: NodeId,
        desired: &ViewFragment,
    ) -> Result<(TransactionCommit, ReconciliationPlan), TransactionError> {
        // 1. Stage the normalized intent: re-render this component's subtree.
        let mut staged = UiTransaction::new();
        staged.push(UiOperation::Rerender { node });

        // 2. Validate before application. On rejection nothing is mutated.
        self.validator.validate(&staged)?;

        // 3. Apply: the subtree-scoped reconcile is the underlying mechanism.
        let plan = reconciler.reconcile(tree, node, desired);

        // 4. Commit atomically, on the same terms as a root commit.
        if plan.is_empty() {
            return Ok((TransactionCommit::new(Vec::new(), self.version), plan));
        }
        self.version = self.version.bump();
        Ok((
            TransactionCommit::new(staged.into_operations(), self.version),
            plan,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_view::view::{ViewFragment, ViewNode};

    fn sample_view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10)),
            ViewNode::element(ComponentId::from_raw(11)),
        ]))
    }

    #[test]
    fn tree_change_commits_through_the_boundary_and_advances_version() {
        let reconciler = Reconciler::new();
        let mut enactor = ReconcileEnactor::new();
        let mut tree = RetainedTree::new();
        let view = sample_view();

        assert_eq!(enactor.version(), VersionStamp::from_raw(0));

        // The first frame populates the tree: a non-empty commit is produced and
        // the version stamp advances — the mutation flowed through the commit.
        let (commit, _plan) = enactor
            .commit_root(&reconciler, &mut tree, &view)
            .expect("a single ReplaceRoot is structurally valid");
        assert!(!commit.is_empty(), "a tree change produced a real commit");
        assert!(commit.is_root_replacement());
        assert_eq!(commit.version(), VersionStamp::from_raw(1));
        assert_eq!(enactor.version(), VersionStamp::from_raw(1));

        // The reconcile mutated the tree exactly as `reconcile_root` would: the
        // synthetic root gained the two element children.
        let root = tree
            .root()
            .expect("root exists after the committed reconcile");
        assert_eq!(tree.get(root).unwrap().children().len(), 2);
    }

    #[test]
    fn idempotent_reconcile_commits_empty_and_holds_the_version() {
        let reconciler = Reconciler::new();
        let mut enactor = ReconcileEnactor::new();
        let mut tree = RetainedTree::new();
        let view = sample_view();

        let _ = enactor.commit_root(&reconciler, &mut tree, &view).unwrap();
        let after_first = enactor.version();

        // Re-committing the same view is a no-op reconcile: an empty commit, and
        // the version stamp does not advance.
        let (second, _plan) = enactor
            .commit_root(&reconciler, &mut tree, &view)
            .expect("still structurally valid");
        assert!(second.is_empty(), "an idempotent reconcile commits empty");
        assert_eq!(second.version(), after_first);
        assert_eq!(enactor.version(), after_first);
    }

    #[test]
    fn committed_tree_matches_a_direct_reconcile() {
        // The commit boundary must not change what the tree becomes: enacting the
        // reconcile through a commit yields the same structure as calling
        // `reconcile_root` directly.
        let reconciler = Reconciler::new();
        let view = sample_view();

        let mut direct = RetainedTree::new();
        let direct_plan = reconciler.reconcile_root(&mut direct, &view);

        let mut enacted = RetainedTree::new();
        let (commit, _plan) = ReconcileEnactor::new()
            .commit_root(&reconciler, &mut enacted, &view)
            .unwrap();

        // Same "did it change?" verdict and the same resulting child structure.
        assert_eq!(!direct_plan.is_empty(), !commit.is_empty());
        let direct_root = direct.root().unwrap();
        let enacted_root = enacted.root().unwrap();
        assert_eq!(
            direct.get(direct_root).unwrap().children().len(),
            enacted.get(enacted_root).unwrap().children().len(),
        );
    }
}
