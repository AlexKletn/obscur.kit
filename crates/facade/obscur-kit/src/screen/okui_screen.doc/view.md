Lowers the compiled document to this frame's [`View`].

Call once per frame — this is the body of the builder
`App::screen` hands
[`UiRuntime::set_root_view`](crate::runtime::UiRuntime::set_root_view).

# What the lowering adds to the bridge

[`UiIrViewBridge`] owns every naming decision (tags → `ComponentId`s,
reserved attributes, events, keys, `<VirtualFor>` markers) and this does not
re-implement any of it: each node's lowering *is* the bridge's. Two things
the bridge cannot know are repaired afterwards, because it has no values to
resolve a binding with:

- a body `{expr}` — which the bridge lowers to an anonymous box that draws
  nothing — becomes a text node carrying the bound value;
- a bound property (`class={theme}`), which the bridge lowers to its *path*
  as placeholder text, takes the bound value verbatim. Verbatim means a
  `class` wants a [`Text`](ViewValue::Text): the value is not stringified,
  because a property is typed data and an `Int` bound to a numeric attribute
  should stay an `Int`.

An unbound path keeps the bridge's placeholder — the path itself — so a
misspelled binding is visible on screen instead of blanking silently.

# `<VirtualFor>` is deliberately left alone

A `<VirtualFor>` lowers to one row *template* that the runtime clones per
realized row, so a value substituted into it would render identically in
every row — the one thing a list must not do. Per-row data comes from
`App::virtual_list`'s Rust row builder,
which is called with the row's model index; a `{binding}` inside a
`<VirtualFor>` therefore keeps its placeholder.
