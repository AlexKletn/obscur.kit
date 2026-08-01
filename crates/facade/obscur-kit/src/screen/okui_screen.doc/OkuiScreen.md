A `.okui` source compiled **once** to UI IR and lowered to a [`View`] on every
frame, resolving its `{bindings}` against a [`ScreenData`] table (spec §2.9).

# What this replaces

`App::screen` used to compile the source in `run` and
hand the runtime a builder that returned that one pre-built `View` for the
application's whole life:

```ignore
let view = compile_screen(&src)?;
Box::new(move || view.clone())      // "static v1 materialization"
```

so no state change could reach the screen — §2.9 defines `.okui` as a front end
"compiled into the same UI IR as the Rust DSL", a *peer* of the Rust DSL rather
than a static subset of it. Holding the IR instead of the `View` costs one
lowering per frame and no parsing at all, and gives the lowering somewhere to
read state from.

# What the two `.okui` routes now do

- **Build time** (`build.rs` +
  [`ViewCodegen`](crate::okui_compiler::codegen::ViewCodegen) with
  [`CodegenRoot::facade`](crate::okui_compiler::codegen::CodegenRoot::facade)):
  the document becomes ordinary Rust, `{count}` becomes a read of the
  application's own `count`, and §2.13's contract validation happens while the
  crate compiles. This is the route §2.9's "compile-time contract validation"
  describes, and it is the one to use for anything with real data behind it.
- **Run time** (this type): the source is text until `run`, so its contract
  errors are a [`ScreenError`] at startup rather than a compile error, and its
  bindings resolve against [`ScreenData`] instead of Rust locals. Structure is
  fixed at compile time — a `<If>`/`<For>` is statically materialised by the
  bridge — so this route reaches *values*, not shape.

# Cost

Lowering is O(nodes) and allocates the frame's `View`; nothing re-parses and
nothing re-compiles. Subtrees that contain neither a bound property nor a body
`{…}` are handed to [`UiIrViewBridge`] whole and never walked twice, so a mostly
static screen costs exactly one bridge lowering — see
[`dynamic_node_count`](Self::dynamic_node_count).
