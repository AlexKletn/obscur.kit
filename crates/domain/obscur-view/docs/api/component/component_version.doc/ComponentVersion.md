A small, ordered version stamp for a registered component's contract schema.

Spec §2.14 lists a "version" among the facts a registry entry records. This
versions the *component/contract schema itself* — independently of the kit
version — so tooling and the plugin system can reason about compatibility of
a component kind as its contract evolves.

It is `Copy` and totally ordered: the derived ordering is major-then-minor,
so `1.0 < 1.5 < 2.0`.
