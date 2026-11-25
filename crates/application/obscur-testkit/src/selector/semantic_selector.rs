//! Retained nodes chosen by what they are, not where they are.

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::NodeId;
use obscur_runtime::UiRuntime;
use obscur_tree::node::NodeKind;

#[doc = include_str!("../../docs/api/selector/semantic_selector.doc/SemanticSelector.md")]
#[derive(Debug)]
pub struct SemanticSelector;

impl SemanticSelector {
    /// Finds nodes by role and optional accessible name in the runtime's semantic
    /// projection. The name is matched exactly and hidden nodes are excluded.
    #[must_use]
    pub fn by_role(runtime: &UiRuntime, role: SemanticRole, name: Option<&str>) -> Vec<NodeId> {
        runtime
            .semantic_tree()
            .by_role(role, name)
            .map(|(id, _)| id)
            .collect()
    }
    /// The number of retained nodes carrying `class`.
    #[must_use]
    pub fn count_class(runtime: &UiRuntime, class: &str) -> usize {
        runtime
            .tree()
            .iter()
            .filter(|(_, node)| node.attributes().has_class(class))
            .count()
    }

    #[doc = include_str!("../../docs/api/selector/semantic_selector.doc/child_text.md")]
    #[must_use]
    pub fn child_text(runtime: &UiRuntime, node: NodeId) -> Option<String> {
        let child = *runtime.tree().get(node)?.child_nodes().first()?;
        match runtime.tree().get(child).expect("live").kind() {
            NodeKind::Text { content } => Some(content.to_string()),
            other => panic!("expected a text node, got {other:?}"),
        }
    }

    #[doc = include_str!("../../docs/api/selector/semantic_selector.doc/first_text.md")]
    #[must_use]
    pub fn first_text(runtime: &UiRuntime) -> String {
        runtime
            .tree()
            .iter()
            .find_map(|(_, node)| node.kind().text().map(str::to_owned))
            .expect("the retained tree has a text node after a frame")
    }
}
