use std::fmt;

use super::NodeInspection;

/// A full structured snapshot of the live tree — one [`NodeInspection`] per node
/// in pre-order — for an agent to observe and assert on in a headless run
/// (spec §7.6). Its [`Display`](fmt::Display) renders the indented tree.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TreeInspection {
    /// Every node, in pre-order (root first).
    pub nodes: Vec<NodeInspection>,
}

impl TreeInspection {
    /// The number of nodes in the snapshot.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the snapshot is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl fmt::Display for TreeInspection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, node) in self.nodes.iter().enumerate() {
            if index > 0 {
                f.write_str("\n")?;
            }
            write!(f, "{node}")?;
        }
        Ok(())
    }
}
