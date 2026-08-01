A `<VirtualFor>` marker becomes a window of realized rows (spec §4.9).

One test, and the count is the honest state of the theme rather than an
oversight: the rest of the virtualization coverage moved into
`runtime/virtual_lists.rs` with the entity that owns it, and what is left here is
the one question that is about the *runtime* rather than about the window
bookkeeping — a marker in a view becomes real nodes in the retained tree, sized
against the committed viewport and not against the collection.

It is also the one test in the suite that needs
[`SemanticSelector`](obscur_testkit::selector::SemanticSelector) rather than a
[`NodePath`](obscur_testkit::selector::NodePath): a realized window has however
many rows the viewport bought, at depths nothing has promised, so the nodes have
to be selected by *what they are*.
