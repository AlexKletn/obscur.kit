Symbol of the authored `options` attribute — a comma-separated list of labels.

The value is `name_binding::attr_symbol("options")`, restated here because the
`obscur-ui-ir` registry that computes it is a frontend crate this
components-tier crate may not depend on. The same duplication (and the same
remedy: a drift test in the one crate that sees both sides) already exists for
`CheckboxProps::LABEL`. The pin is `obscur-kit`'s `tests/builtin_components.rs`.
