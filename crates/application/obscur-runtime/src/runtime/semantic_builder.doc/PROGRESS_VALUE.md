The completed fraction a progress bar carries, MIRRORED from
`obscur_widgets::progress::Progress::PROP_VALUE`.

A mirror rather than a use, because `obscur-widgets` sits ABOVE this crate
in the layering and a dependency on it would be a cycle in the graph
`xtask check` enforces. The same mirror already exists in
`obscur_ui_ir::name_binding::PROGRESS_VALUE`, which is what compiles
`<progress value=…/>` markup to this property.

The hazard the mirror buys: nothing in this crate can fail to compile if
the widget renumbers its property, and a renumbered widget would go silent
to assistive technology without a word. The fix that removes the hazard is
to mint the symbol in `obscur-view` — the domain vocabulary both sides can
name — exactly as `SLIDER_LOW` is minted, and have the widget use it.
