A stable, opaque identifier for a compiled selector.

Selectors are parsed and compiled by `obscur-css`; the concrete selector
syntax and the matching engine (combinator walking, `:nth-*`, attribute
tests, pseudo-state requirements) live there, not in this crate. The
cascade references selectors by this id — to order declarations or to look
up match results — without re-parsing or inspecting selector internals.

The backing integer is a handle minted by the compiler; it carries no
meaning outside that subsystem and must never be treated as a public
schema.
