Records `node`'s place in the pre-order walk: its `tree_index` (the index
its own paint and every `Push*` of its brackets sort at), its `parent` and
its `depth`.

Every node the walk visits is entered, whether or not it goes on to record a
bracket — a descendant with no bracket of its own still pushes its ancestor's
exit index outward, which is what makes an ancestor's `Pop*` sort after it.
