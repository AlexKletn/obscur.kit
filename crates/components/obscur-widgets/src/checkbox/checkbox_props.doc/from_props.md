Decodes a `<checkbox>` node's properties.

`label` is **required**, mirroring `widget_contracts()`'s
`requiring_prop("label")` — the §2.13 statement the `.okui` compiler already
enforces for `<Checkbox/>`. A labelless node therefore fails to decode, the
error boundary reports it, and the control renders nothing: visible and
diagnosable, rather than a blank box with no explanation.

`checked` is read from **two** sources, in order, because two front ends
state it two different ways:

1. `Checkbox::PROP_CHECKED`, the widget's own tri-state integer
   (`-1` mixed, `0` off, `1` on). Only [`Checkbox::view`] emits it, and it
   is the only spelling that can say *mixed*, so it wins where present.
   This is what makes mounting idempotent on the Rust-API path.
2. Otherwise the reserved `CHECKED` attribute, through
   [`StyleAttributes`] — the *same* reader that decides the `:checked`
   pseudo-class. That matters: an authored `<checkbox checked/>` lowers to
   `ViewValue::Null`, which the reserved-attribute rule reads as set
   ("presence = set") while a plain `Option<bool>` decode would read as
   absent. Going through `StyleAttributes` makes it impossible for the
   painted indicator and the matched `:checked` to disagree.

Markup has only the two-state spelling; mixed is a Rust-API state.

[`Checkbox::view`]: super::Checkbox::view
[`StyleAttributes`]: obscur_view::view::StyleAttributes
