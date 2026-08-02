Symbol of the authored `value` attribute.

The value is `name_binding::attr_symbol("value")` — the symbol both front
ends put on an authored `value="…"` — restated here because the
`obscur-ui-ir` registry that computes it is a frontend crate this
components-tier crate may not depend on. The pin is `obscur-kit`'s
`tests/builtin_components.rs`.

Note which symbol this is **not**: the reserved `VALUE`
(`obscur_view::view::VALUE`, `0x0000_0008`) that [`Input::view`] also emits
declares a *controlled* value, which the runtime snaps a drifting buffer back
to. Markup's `value="…"` is HTML's **default** value — it seeds the field and
then the user owns it — so the two must not be one symbol.

[`Input::view`]: super::Input::view
