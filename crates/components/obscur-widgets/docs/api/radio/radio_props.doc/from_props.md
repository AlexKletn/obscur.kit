Decodes a `<radio>` node's properties.

`label` is **required**, mirroring `widget_contracts()`'s
`requiring_prop("label")` — the §2.13 statement the `.okui` compiler already
enforces for `<Radio/>`. A labelless node therefore fails to decode, the error
boundary reports it, and the control renders nothing: visible and diagnosable,
rather than a blank dot with no explanation.

`selected` is read from **two** sources, in order, because two front ends
state it two different ways:

1. [`Radio::PROP_SELECTED`], the widget's own boolean. Only [`Radio::view`]
   emits it, and it is emitted on *every* node the builder makes — including
   the unselected ones, where the reserved attribute is deliberately absent.
   Reading it first is what makes mounting idempotent on the Rust-API path.
2. Otherwise the reserved `CHECKED` attribute, through [`StyleAttributes`] —
   the *same* reader that decides the `:checked` pseudo-class. That matters:
   an authored `<radio checked/>` lowers to `ViewValue::Null`, which the
   reserved-attribute rule reads as set ("presence = set") while a plain
   `bool` decode would read as absent. Going through [`StyleAttributes`] makes
   it impossible for the painted dot and the matched `:checked` to disagree.

[`Radio::PROP_SELECTED`]: super::Radio
[`Radio::view`]: super::Radio::view
[`StyleAttributes`]: obscur_view::view::StyleAttributes
