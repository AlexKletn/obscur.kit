#![doc = include_str!("reconcile_enactor.doc/_module.md")]

use obscur_identity::{NodeId, VersionStamp};
use obscur_tree::reconciliation::{Reconciler, ReconciliationPlan};
use obscur_tree::tree::RetainedTree;
use obscur_view::view::{View, ViewFragment};

use super::transaction_commit::TransactionCommit;
use super::transaction_error::TransactionError;
use super::transaction_validator::TransactionValidator;
use super::ui_operation::UiOperation;
use super::ui_transaction::UiTransaction;

#[doc = include_str!("reconcile_enactor.doc/ReconcileEnactor.md")]
#[derive(Debug, Clone, Default)]
pub struct ReconcileEnactor {
    #[doc = include_str!("reconcile_enactor.doc/validator.md")]
    validator: TransactionValidator,
    #[doc = include_str!("reconcile_enactor.doc/version.md")]
    version: VersionStamp,
}

impl ReconcileEnactor {
    #[doc = include_str!("reconcile_enactor.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            validator: TransactionValidator::new(),
            version: VersionStamp::from_raw(0),
        }
    }

    #[doc = include_str!("reconcile_enactor.doc/version_2.md")]
    #[must_use]
    pub const fn version(&self) -> VersionStamp {
        self.version
    }

    #[doc = include_str!("reconcile_enactor.doc/validator_2.md")]
    #[must_use]
    pub const fn validator(&self) -> &TransactionValidator {
        &self.validator
    }

    #[doc = include_str!("reconcile_enactor.doc/commit_root.md")]
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

    #[doc = include_str!("reconcile_enactor.doc/commit_subtree.md")]
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
        assert_eq!(tree.get(root).unwrap().child_nodes().len(), 2);
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
            direct.get(direct_root).unwrap().child_nodes().len(),
            enacted.get(enacted_root).unwrap().child_nodes().len(),
        );
    }
}
