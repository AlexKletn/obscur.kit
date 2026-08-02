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
because each sets the reserved local-name property itself
(`obscur_widgets::Button::view`: "Stable CSS type / local name so a UA (or
author) `button { … }` type selector matches this control").

Three things changed, two of them in the frontend:

1. `name_binding::local_name_for` — an *element*-position tag is also the
   node's local name, emitted as the reserved `LOCAL_NAME` property by both
   lowering paths (`ViewCodegen::emit_element` and
   `UiIrViewBridge::lower_element`), so the compiled `okui!` path and the
   interpreted `.okui` path agree.
2. every lowercase control tag the UA sheet styles binds to the shipped
   widget's `COMPONENT_ID`, the way `img` and `Image` already were one handle
   — `button`, `input`, `textarea`, `checkbox`, `radio`, `select`, `table`.
   HTML has a `<button>` and an `<input>`; so does this. `<checkbox>` used to
   name no built-in at all, so markup produced a styled `div` while
   `Checkbox::new()` produced the control, and one `checkbox { … }` rule
   reached two different trees. `textarea` is the one spelling with no
   PascalCase twin: HTML's two text-entry tags are one widget here, and which
   of them a node is stays in its local name, where `EditableField::is_multiline`
   already read it.
3. the local name got a symbol of its own. It used to share `TYPE` with HTML's
   authored `type="…"`, and the author's value took the whole slot, so
   `<button type="submit">` — ordinary markup — was an element whose CSS type
   was `submit` and matched no `button { … }` rule at all.

# What an authored `<button>` is, and is not

It **is** an element with the button widget's component id, the author's own
children, the author's `on:click` binding (which the runtime dispatches off the
retained node's event binding, not off any component), the local name
`button` — and therefore the UA sheet's padding/border/background, and the
`SemanticRole::Button` the accessibility tree derives from a local name.

What it is *not* is a node whose children the widget replaces. `KitBootstrap`
does register a `ButtonFactory` for that id by default now, and the reason the
sentence above still says "the author's own children" is that
`ButtonComponent`'s whole render is the §2.12 **default-slot outlet**: the
runtime projects those children straight back into it, and `label="…"` is only
the fallback for markup that supplies none. The tests in this file build views
and read the desired tree, so they see the authored node either way — what
mounts under it is `builtin_components.rs`'s subject, not this file's.
