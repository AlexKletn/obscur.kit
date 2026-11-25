The reserved symbol an attribute binds to on a *control* tag, or `None` when the
tag and name name no control attribute.

The pairs are tag-aware for the same reason `(slot, name)` is: `value` means the
field's text on an `<input>` — where it stays hashed on purpose, because the
reserved `obscur_view::view::VALUE` declares a *controlled* value the runtime
snaps a drifting buffer back to — and it means the thumb's position on a
`<slider>`. One tag-blind table could hold only one of those meanings.
