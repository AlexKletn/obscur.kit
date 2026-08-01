#![doc = include_str!("semantic_selector.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_runtime::UiRuntime;
use obscur_tree::node::NodeKind;

#[doc = include_str!("semantic_selector.doc/SemanticSelector.md")]
#[derive(Debug)]
pub struct SemanticSelector;

impl SemanticSelector {
    #[doc = include_str!("semantic_selector.doc/count_class.md")]
    #[must_use]
    pub fn count_class(runtime: &UiRuntime, class: &str) -> usize {
        runtime
            .tree()
            .iter()
            .filter(|(_, node)| node.attributes().has_class(class))
            .count()
    }

    #[doc = include_str!("semantic_selector.doc/child_text.md")]
    #[must_use]
    pub fn child_text(runtime: &UiRuntime, node: NodeId) -> Option<String> {
        let child = *runtime.tree().get(node)?.child_nodes().first()?;
        match runtime.tree().get(child).expect("live").kind() {
            NodeKind::Text { content } => Some(content.to_string()),
            other => panic!("expected a text node, got {other:?}"),
        }
    }

    #[doc = include_str!("semantic_selector.doc/first_text.md")]
    #[must_use]
    pub fn first_text(runtime: &UiRuntime) -> String {
        runtime
            .tree()
            .iter()
            .find_map(|(_, node)| node.kind().text().map(str::to_owned))
            .expect("the retained tree has a text node after a frame")
    }
}
