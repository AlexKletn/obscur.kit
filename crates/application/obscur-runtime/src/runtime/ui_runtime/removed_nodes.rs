#![doc = include_str!("removed_nodes.doc/_module.md")]

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("removed_nodes.doc/drop_state_of_removed_nodes.md")]
    pub(super) fn drop_state_of_removed_nodes(&mut self) {
        // The layout side has needed none of this: `arrange` replaces its whole
        // map from the pass output, so it is the live tree by construction. The
        // style side retains on purpose — that is what makes it incremental —
        // and retention is only ever half a policy until something states what
        // leaves.
        self.computed.retain(|node, _| self.tree.contains(*node));
        self.inputs.retain_live(&self.tree);
        self.pseudo.retain_live(&self.tree);
        self.scroll.retain_live(&self.tree);
    }
}
