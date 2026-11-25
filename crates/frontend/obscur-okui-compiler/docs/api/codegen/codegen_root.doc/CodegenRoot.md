The two absolute paths [`ViewCodegen`](super::ViewCodegen) writes its emitted
types with: the module holding `obscur-view`'s view types, and the crate
holding `obscur-identity`'s handles.

# Why this is an argument and not a constant

It used to be a constant — the emitted paths were the literals
`::obscur_view::view::…` and `::obscur_identity::…` — and that made the
primary authoring surface unusable outside this workspace. An application
depends on the `obscur-kit` facade and nothing else, exactly as
`docs/api/README.md` tells it to; the facade re-exports those crates under
*different* names (`obscur_kit::view`, `obscur_kit::identity`), so the first
`okui!` in an application failed with

```text
error[E0433]: cannot find `obscur_view` in the crate root
```

A `::`-prefixed path resolves in the **extern prelude**, which holds a
crate's *direct* dependencies only, so re-exporting under the expected names
could not have fixed it: no re-export puts `obscur_view` there. The emitted
root has to be the caller's, so the caller states it.

That the defect survived to be found from outside is structural: every
in-repo consumer of the codegen lives *inside* `obscur-kit`, where both
crates are direct dependencies, so the failing path was never taken. The
argument is modelled on
[`TagPosition`](obscur_ui_ir::name_binding::TagPosition) for the same reason
it exists there — a caller must *state* which resolution it wants, so the
wrong one can no longer be picked by omission.

# Which one to pass

[`facade`](Self::facade) for anything an application compiles: its own
`okui!` call sites and its `build.rs`-generated files alike, since
`obscur-kit` is the one crate it depends on. [`internal`](Self::internal)
only for a crate that depends on `obscur-view` and `obscur-identity`
directly. [`new`](Self::new) for a graph that re-exports them somewhere else
again.
