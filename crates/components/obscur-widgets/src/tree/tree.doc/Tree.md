A headless hierarchical tree with expansion and single selection (spec §2.6).

`Tree` owns a flat, pre-order [`Vec`] of [`TreeNode`]s (see that type for the
depth-based hierarchy encoding), the set of currently
[`expanded`](Self::is_expanded) parent indices, and the single
[`selected`](Self::selected_index) node index (if any). It bakes in **no**
visual theme: [`view`](Self::view) derives an element carrying only the
**visible** nodes (those whose every ancestor is expanded), each as a
treeitem-shaped child element carrying its depth, expanded flag (parents
only), selected flag, and a [`Text`] label child, rendered by the style layer
keyed on [`COMPONENT_ID`](Self::COMPONENT_ID).

The accessible role is [`SemanticRole::List`]: [`SemanticRole`] has no
dedicated tree role, and a list is the closest available classification.

The state machine covers expansion ([`expand`](Self::expand),
[`collapse`](Self::collapse), [`toggle_expanded`](Self::toggle_expanded), all
no-ops on leaf nodes) and single selection ([`select`](Self::select)).
