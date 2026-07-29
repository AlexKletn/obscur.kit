# 09 — Authoring formats

Three ways to write a view. All three end at the same place: ordinary Rust
building a `View`. **There is no interpreter at runtime** — no IR, no `.okui`
machinery and no document survives compilation.

| | |
|---|---|
| Rust builders | always available; everything else lowers to this |
| `okui! { … }` | the primary authoring surface, a proc-macro |
| `.okui` files | the fallback, compiled by a build step |

---

## The `okui!` macro

`obscur_kit::ui_macros::okui!` parses a TSX-like tree, lowers it to the UI IR and
runs it through `ViewCodegen` **during expansion**, so what lands in your crate
is plain Rust.

```rust
let view = okui! {
    <container class="row">
        <Button label="OK" on:click={on_ok}/>
        "Ready"
    </container>
};
```

Also exported: `#[derive(Contract)]`, which emits a component's
`contract_surface()` for §2.13 validation.

**What your crate needs:** `obscur-kit`, and nothing else. The expansion is
self-contained — every path in it is absolute and written through the facade
(`::obscur_kit::view::view::…`, `::obscur_kit::identity::…`), so no `use` item and
no second dependency is required. Until `c9f7368` this was untrue: the expansion
named `::obscur_view` and `::obscur_identity`, which resolve only in a crate that
depends on them *directly*, so the first `okui!` in an application failed with
`error[E0433]: cannot find obscur_view in the crate root`. A build step chooses
the same root explicitly — `ViewCodegen::document_to_view_fn(ir, name,
&CodegenRoot::facade())`.

`checks/facade-consumer` is a crate outside the workspace whose only dependency
is the facade; `cargo run -p xtask -- consumer` builds it. It exists because no
in-repo consumer stands on that side of the boundary, which is why the defect
above survived as long as it did.

## `.okui` files

```rust
App::new("Showcase").screen(include_str!("showcase.okui")).run()
```

`App::screen` compiles the markup at `run`, returning `AppError::ScreenParse` or
`AppError::ScreenCompile` with one message per diagnostic. A build step can do
the same ahead of time through `obscur_okui_compiler`.

```html
<app id="app" class="app">
  <header class="header">
    <span class="title">Title</span>
  </header>
  <div class="gallery">
    <div type="card" class="solid">…</div>
  </div>
</app>
```

Attributes map onto the reserved view properties: `class`, `id`, `type`, `src`,
and `on:<event>={handler}` onto a `ViewEventBinding`.

## Known tags

**Primitives:** `Canvas`, `Container`, `ForeignSurface`, `Image`, `Layer`,
`Portal`, `Scroll`, `Text`.

**Widgets:** `Button`, `Checkbox`, `Dialog`, `Input`, `Radio`, `Select`.

**Control flow:** `<For>`, `<If>`, `<Else>`, `<Empty>`, `<Slot>`.

Anything else is an unknown surface and is refused with a diagnostic rather than
being emitted blindly.

Note the widget list is **six**, not the thirteen in
[05 — Components](05-components.md): `List`, `Table`, `Tabs`, `Tree`, `Menu` and
`Docking` have no markup contract, so they are reachable only from Rust.

## Validation

Both front ends enforce §2.13 component contracts from the same registry
(`ContractRegistry::builtins()`), so a typo in an attribute name is a compile
error rather than a silently ignored property.

Body interpolations are validated on the same allow-list as attributes — a
`<span>{expr}</span>` is lowered to a real binding and emitted as real Rust, so
it is checked, not passed through.

## What is refused

- **`<VirtualFor>`** — refused on both front ends. §4.9 windowing from markup is
  not implemented; use `App::virtual_list` / `UiRuntime::register_virtual_list`.
- **User components** — `#[derive(Contract)]` and `ContractRegistry::with_surface`
  exist, but no component can be *authored* in either front end yet, and the
  registration seam has no in-repo caller by construction: a `build.rs` cannot
  depend on its own crate, and the proc-macro has no argument to be handed one.
  The downstream wire is real; this repo cannot demonstrate it.
- **`<slot>` in a user component's render** — the projection machinery works, but
  see the previous point for who could use it.

## The UI IR

`obscur_kit::ui_ir` is the shared middle: both front ends lower into it, and
`ViewCodegen` lowers it to Rust. Its `name_binding` table is what keeps event
names (`"click"`, `"input"`, `"change"`, `"longpress"`) in step with the
`EVENT_*` symbols in `obscur-view`; a drift test pins the two together.

You would touch the IR only to write a third front end.
