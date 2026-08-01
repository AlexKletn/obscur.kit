End-to-end: an authored tag is a **base element** — it carries its own local
name, so a §4.13 type selector matches it — and `<button>` is one of them
(spec §4.13, §2.8).

# What this closes

An authored element used to reach the tree carrying a `ComponentId` and
nothing else, and a `ComponentId` is not a selector input: no part of matching
reads one. So `<button>`, `<div>` and `<td>` were three *anonymous* boxes, and
every type-selector rule in existence — including the whole of the facade's UA
stylesheet, which is written in `button { … }` / `input { … }` / `td { … }` and
nothing else — matched none of them. Only the Rust widgets were styleable,
because each sets the reserved `TYPE` property itself
(`obscur_widgets::Button::view`: "Stable CSS type / local name so a UA (or
author) `button { … }` type selector matches this control").

Two things changed, both in the frontend:

1. `name_binding::local_name_for` — an *element*-position tag is also the
   node's local name, emitted as the reserved `TYPE` property by both lowering
   paths (`ViewCodegen::emit_element` and `UiIrViewBridge::lower_element`), so
   the compiled `okui!` path and the interpreted `.okui` path agree.
2. the lowercase `button` tag binds to `Button::COMPONENT_ID`, the way `img`
   and `Image` already were one handle. HTML has a `<button>`; so does this.

# What an authored `<button>` is, and is not

It **is** an element with the button widget's component id, the author's own
children, the author's `on:click` binding (which the runtime dispatches off the
retained node's event binding, not off any component), the local name
`button` — and therefore the UA sheet's padding/border/background, and the
`SemanticRole::Button` the accessibility tree derives from a local name.

It is **not** a live `obscur_widgets::Button`: nothing registers a factory for
the built-in widget ids by default (`UiRuntime::components` is "empty and inert
until a host calls `register_component`"), so the widget's own view — its
pressed state, its synthesized label child, its `:disabled` attribute — does
not run. The id is the handle a host *could* register under; it is not a
mounting.
