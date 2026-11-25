Lowers the compiled document to this frame's [`View`].

Call once per frame — this is the body of the builder
`App::screen` hands
[`UiRuntime::set_root_view`](crate::runtime::UiRuntime::set_root_view).

# What the lowering adds to the bridge

[`UiIrViewBridge`](crate::okui_compiler::bridge::UiIrViewBridge) owns every naming decision (tags → `ComponentId`s,
reserved attributes, events, keys, `<VirtualFor>` markers) and this does not
re-implement any of it: each node's lowering *is* the bridge's. Two things
the bridge cannot know are repaired afterwards, because it has no values to
resolve a binding with:

- a body `{expr}` — which the bridge lowers to an anonymous box that draws
  nothing — becomes a text node carrying the bound value;
- a bound property (`class={theme}`), which the bridge lowers to its *path*
  as placeholder text, takes the bound value verbatim. Verbatim means a
  `class` wants a [`Text`](crate::view::view::ViewValue::Text): the value is not stringified,
  because a property is typed data and an `Int` bound to a numeric attribute
  should stay an `Int`.

An unbound path keeps the bridge's placeholder — the path itself — so a
misspelled binding is visible on screen instead of blanking silently.

# Runtime collections

Bind records with [`ScreenData::set_rows`] before compiling the screen. A
`VirtualFor` can then read fields such as `row.label` and use a text or integer
field such as `row.id` as its stable key. Each view captures the current
bindings, and only the rows in the realized window are lowered.
