Symbol of the authored `label` attribute.

The value is `name_binding::attr_symbol("label")` — the symbol both front
ends put on an authored `label="…"`, and the same one
[`CheckboxProps::LABEL`] carries, because it is the same attribute name.
It is restated here because the `obscur-ui-ir` registry that computes it is a
frontend crate this components-tier crate may not depend on; the remedy for
the duplication is the drift test in the one crate that sees both sides,
`obscur-kit`'s `tests/builtin_components.rs`.

[`CheckboxProps::LABEL`]: crate::checkbox::CheckboxProps::LABEL
