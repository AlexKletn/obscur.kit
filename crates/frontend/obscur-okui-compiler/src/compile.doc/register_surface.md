Registers a user component's string [`ComponentSurface`] (bridged into a
string [`ComponentContract`]), so the `.okui` compiler enforces §2.13 for
that component just as it does for the built-ins.

A caller wires user components in explicitly, e.g.
`ContractRegistry::builtins().with_surface(&MyProps::contract_surface())`,
where `contract_surface()` is emitted by `#[derive(Contract)]`. Start from
[`builtins`](Self::builtins), not [`new`](Self::new): `Compiler::with_registry`
**replaces** its registry wholesale, so seeding from an empty one silently
drops every built-in primitive/widget contract.

Returns the contract it displaced for the same component name, if any.

# Where this is callable, and where it is not

This crate has no in-repo production caller and cannot have one: a caller
needs a *user* component, and this repo ships none — every component it
ships is a built-in whose surface is already mirrored into
[`builtin_contracts`]/[`widget_contracts`] and registered by
[`Compiler::new`] unconditionally. The other end of this seam lives in a
downstream crate.

Which downstream callers can reach it is **not** a free choice, and the
constraint is easy to state backwards. `contract_surface()` is a function
that runs when the crate defining it runs, so nothing that *compiles* that
crate can call it:

- A `build.rs` compiling a `.okui` file **cannot** see a component defined
  in its own crate — `[build-dependencies] X` inside `X` is rejected by
  cargo as a `cyclic package dependency`. It *can* see one from any other
  crate, and that is the build-path wire.
- The `okui!` proc-macro cannot see one at all, and has no argument to be
  handed one through. See `obscur_ui_macros`'s `validate_contracts`.
- A **running binary** that links the component crate — the `.okui`
  language server is the one in this repo — can call it freely. That is
  the least constrained consumer of this function, and §2.14's "static
  registration is the default" is reachable there in full.

There is no auto-discovery, and inventory/link-time collection cannot
supply one on the build path (it collects when the *built* binary runs,
which is after the build script it would have to serve). A build-time
registry would have to re-parse the crate's own Rust source — the surface
is a pure function of the derive's tokens — which nothing here proposes.

[`builtin_contracts`]: crate::validation::builtin_contracts
[`widget_contracts`]: crate::validation::widget_contracts
