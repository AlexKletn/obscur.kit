Lowers the field into its string-surface builder call
(`.requiring_prop(...)` / `.allowing_prop(...)` / `.allowing_event(...)`),
or `None` when the field is skipped.

The name is emitted as the raw string literal (before it is hashed for
the typed contract), so a string-keyed tool can match it against `.okui`
attribute/event names. Required vs. allowed mirrors
[`contract_tokens`](Self::contract_tokens): an `Option<...>` or
`#[contract(default)]` prop is allowed (optional); every other prop is
required; an event field becomes an allowed event.
