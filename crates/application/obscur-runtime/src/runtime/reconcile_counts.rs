use obscur_tree::reconciliation::{ReconciliationOp, ReconciliationPlan};

#[derive(Default)]
pub(crate) struct ReconcileCounts {
    pub(super) inserted: usize,
    pub(super) removed: usize,
    pub(super) moved: usize,
    pub(super) updated: usize,
}

impl ReconcileCounts {
    pub(super) fn record(&mut self, plan: &ReconciliationPlan) {
        for op in plan.ops() {
            match op {
                ReconciliationOp::Insert { .. } => self.inserted += 1,
                ReconciliationOp::Remove { .. } => self.removed += 1,
                ReconciliationOp::Move { .. } => self.moved += 1,
                ReconciliationOp::Update { .. } => self.updated += 1,
            }
        }
    }
}
