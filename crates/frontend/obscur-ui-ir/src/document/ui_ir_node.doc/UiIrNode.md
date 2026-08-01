One node of the UI IR tree, carrying a compiler-assigned stable id
(spec §2.5).

The IR tree mirrors the runtime view tree but is the serializable compile
artifact: every node has a stable [`id`](UiIrNode::id) (used by source maps
and diagnostics), a [`kind`](UiIrNode::kind), and — depending on kind — a
tag, text content, an optional stable [`key`](UiIrNode::key) (mandatory on a
[`Loop`](UiIrNodeKind::Loop) lowered from a `<For>`, spec §2.11), properties
flowed down, event bindings flowing up, and recursively nested children.
