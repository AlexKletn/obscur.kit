Symbol of the authored `label` attribute.

The value is `name_binding::attr_symbol("label")` — the symbol both front ends
put on an authored `label="…"` — restated here because the `obscur-ui-ir`
registry that computes it is a frontend crate this components-tier crate may
not depend on. It is the same value [`CheckboxProps::LABEL`] carries, for the
same reason and with the same remedy: a drift test in the one crate that sees
both sides, `obscur-kit`'s `tests/builtin_components.rs`.

[`CheckboxProps::LABEL`]: crate::checkbox::CheckboxProps::LABEL
