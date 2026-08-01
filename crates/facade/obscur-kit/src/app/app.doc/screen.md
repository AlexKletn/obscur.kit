Sets the root to an `.okui` screen: parsed and compiled to UI IR when
[`run`](Self::run) is called, then lowered to a [`View`] on **every frame**,
with `{bindings}` resolved against [`screen_data`](Self::screen_data). A
compile failure makes `run` return an [`AppError`] carrying the diagnostics.
Overrides any previous [`view`](Self::view) / [`screen`](Self::screen).

# What state can and cannot reach it

This used to materialise the screen **once** and hand the runtime a builder
that returned that one `View` for the application's whole life, so no state
change could reach it at all. Now the compiled IR is kept and re-lowered per
frame (nothing re-parses), which means:

- **values do reach it**: a body `{count}` and a bound attribute
  (`class={theme}`) read [`ScreenData`], so writing that table between frames
  changes what the screen shows. A name an *attribute* binds must be bound in
  [`screen_data`](Self::screen_data) **before** `run`, because that is what
  declares it to the compiler — an undeclared name is a §2.13 error from `run`
  rather than a value that silently never arrives;
- **shape does not**: `<If>`/`<For>` are statically materialised by the
  bridge on this route, so the *structure* is fixed at compile time, and a
  `{binding}` inside a `<VirtualFor>` keeps its placeholder (per-row data
  comes from [`virtual_list`](Self::virtual_list), which is told the row's
  model index);
- **contracts are validated here, not at compile time**: §2.9 asks for
  compile-time contract validation, and a source that is a string until `run`
  cannot have it. Use the **build-time** route for that — a `build.rs` that
  emits through
  [`CodegenRoot::facade`](crate::okui_compiler::codegen::CodegenRoot::facade),
  or the inline `okui!` macro — where the markup becomes ordinary Rust,
  `{count}` reads the application's own binding, and a contract error is a
  compile error. Pass the generated function to [`view`](Self::view).

See [`OkuiScreen`] for the whole comparison between the two routes.
