Bridging compiled UI IR into the runtime desired-view tree (spec §2.15).

[`UiIrViewBridge`] lowers a
[`UiIrDocument`](obscur_ui_ir::document::UiIrDocument) / its
[`UiIrNode`](obscur_ui_ir::document::UiIrNode) tree into an
[`obscur_view`] [`View`](obscur_view::view::View) so the frontend IR and the
reconciler connect end-to-end: a compiled `.okui` document becomes a desired
tree the reconciler turns into a retained tree.
