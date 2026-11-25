The values an [`OkuiScreen`](super::OkuiScreen)'s `{bindings}` resolve against,
shared by handle (spec §2.8, §2.9).

# Why this exists

A `.okui` file is markup, and the two ways to turn it into a view differ in
*where the bindings are resolved*:

- the **build-time** route (`build.rs` →
  [`ViewCodegen`](crate::okui_compiler::codegen::ViewCodegen)) emits ordinary
  Rust, so `{count}` is compiled to a read of the application's own `count`
  binding — full reactivity, and §2.13's contract validation happens while the
  crate is built;
- the **run-time** route (`App::screen` /
  [`OkuiScreen`](super::OkuiScreen)) parses and compiles the source inside
  `run`, where no Rust locals exist to read. The IR keeps a binding as a
  *path*, and the runtime bridge — which cannot evaluate anything — lowers that
  path to placeholder text.

This table is what that second route resolves paths against: an application
binds `"count"` here, the screen reads it while it lowers, and a value written
between two frames is visible on the second. Before it, a `.screen` was
materialised **once** and no state change could reach it at all; the builder
returned the same pre-built `View` on every frame.

# Semantics

A handle is cheap to [`clone`](Clone) and every clone names the same table, so
a command handler can capture one and write to it (a handler receives no
runtime and must not touch one — see
[`AppControl`](crate::control::AppControl) — but this is plain application
state, so writing it needs no deferral). A name with no value bound is not an
error: the screen keeps the bridge's placeholder, so an unbound `{count}`
renders the *path* and is visible rather than silently blank.

Values are [`ViewValue`]s because that is what a property lowers to; text
content renders a value through [`ScreenData::render`], which is how an `Int`
bound to `{count}` becomes the characters `7`.
