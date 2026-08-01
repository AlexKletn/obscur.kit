Reserved symbol for a text control's **placeholder** — the hint shown while the
field is empty (the CSS `::placeholder` pseudo-element, shown while
`:placeholder-shown` holds). Like [`VALUE`] (and unlike `class`/`id`/`type`)
this is **not** a CSS selector input: it is the field's own placeholder string,
carried verbatim (as [`ViewValue::Text`]). A widget emits it so the runtime can
paint the placeholder as a §4.16 synthetic part — paint-only, DevTools-visible,
hit-transparent, and **not** a component-tree child — rather than materialising
it into the tree. Same reserved low-symbol namespace so it survives
reconciliation.
