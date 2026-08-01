Creates a registry pre-populated with the contracts for every built-in
view primitive **and every shipped widget** (spec §2.6, §2.13), so a
default compile enforces their attribute/event surface without the caller
registering anything.

This is the registry [`Compiler::new`] installs by default. See
[`builtin_contracts`](crate::validation::builtin_contracts) and
[`widget_contracts`](crate::validation::widget_contracts) for what each
covers, and why the surfaces are defined here rather than imported from
`obscur-view`/`obscur-primitives`/`obscur-widgets`.
