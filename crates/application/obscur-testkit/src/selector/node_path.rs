//! A node named by its position under the root.

use obscur_identity::NodeId;
use obscur_runtime::UiRuntime;

#[doc = include_str!("../../docs/api/selector/node_path.doc/NodePath.md")]
#[derive(Debug, Clone, Copy)]
pub struct NodePath<'a> {
    /// The child index to take at each depth, starting from the root.
    steps: &'a [usize],
}

impl<'a> NodePath<'a> {
    /// A path over the given child indices.
    #[must_use]
    pub fn new(steps: &'a [usize]) -> Self {
        Self { steps }
    }

    #[doc = include_str!("../../docs/api/selector/node_path.doc/resolve.md")]
    #[must_use]
    pub fn resolve(&self, runtime: &UiRuntime) -> NodeId {
        let tree = runtime.tree();
        let mut node = tree.root().expect("the tree has a root after a frame");
        for (depth, step) in self.steps.iter().enumerate() {
            let children = tree
                .get(node)
                .unwrap_or_else(|| panic!("step {depth} of {:?} names a live node", self.steps))
                .child_nodes();
            node = *children.get(*step).unwrap_or_else(|| {
                panic!(
                    "step {depth} of {:?} asks for child {step} of a node with {} children",
                    self.steps,
                    children.len()
                )
            });
        }
        node
    }

    #[doc = include_str!("../../docs/api/selector/node_path.doc/children.md")]
    #[must_use]
    pub fn children(&self, runtime: &UiRuntime) -> Vec<NodeId> {
        let node = self.resolve(runtime);
        runtime
            .tree()
            .get(node)
            .expect("the resolved node is live")
            .child_nodes()
            .to_vec()
    }
}
