# 02 — The view model

A frame's input is a `View`: a plain, owned description of what should be on
screen. It carries no state, no callbacks and no identity of its own — the
runtime reconciles it against the retained tree and keeps identity there.

The consequence worth internalising: **you rebuild the whole view every frame**
and the runtime works out what changed. Building a view is cheap by design.

```rust
use obscur_kit::view::view::{View, ViewNode, ViewFragment, ViewEventBinding, EVENT_CLICK};
use obscur_kit::identity::{ComponentId, InternedSymbol};

View::single(
    ViewNode::element(ComponentId::from_raw(1))
        .class("row")
        .with_event(ViewEventBinding::new(EVENT_CLICK, InternedSymbol::named("save")))
        .with_children(ViewFragment::from_nodes(vec![ViewNode::text("Save")])),
)
```

---

## `View`

```rust
pub const fn empty() -> Self
pub fn single(node: ViewNode) -> Self
pub fn from_fragment(root: ViewFragment) -> Self
pub const fn root(&self) -> &ViewFragment
pub fn into_fragment(self) -> ViewFragment
pub fn into_single_node(self) -> Option<ViewNode>
```

A view is a fragment at the root, so it may hold several top-level nodes.
`into_single_node` returns `None` when it holds anything but exactly one.

## `ViewFragment`

An ordered list of siblings.

```rust
pub const fn new() -> Self
pub fn from_nodes(nodes: Vec<ViewNode>) -> Self
pub fn push(&mut self, node: ViewNode)
pub fn iter(&self) -> Iter<'_, ViewNode>
pub fn nodes(&self) -> &[ViewNode]
pub fn into_nodes(self) -> Vec<ViewNode>
pub fn len(&self) -> usize
pub fn is_empty(&self) -> bool
```

## `ViewNode`

```rust
pub fn element(component: ComponentId) -> Self
pub fn text(content: impl Into<Box<str>>) -> Self
pub fn slot(name: impl Into<Box<str>>) -> Self
pub fn virtual_list(list_id: InternedSymbol, count: usize, row_extent: f32) -> Self
pub fn virtual_collection(row_extent: f32, template: ViewFragment) -> Self
```

Builders, each returning `Self`:

```rust
pub fn class(self, class: impl Into<Box<str>>) -> Self
pub fn with_key(self, new_key: impl Into<ViewKey>) -> Self
pub fn with_prop(self, prop: ViewProperty) -> Self
pub fn with_event(self, event: ViewEventBinding) -> Self
pub fn with_children(self, new_children: ViewFragment) -> Self
pub fn with_virtual_scroll_reset(self, token: u64) -> Self
pub fn key(&self) -> Option<&ViewKey>
```

`class` is the shorthand you will use constantly; it is `with_prop` against the
reserved `CLASS` symbol. Several classes go in one string, space-separated, as in
CSS: `.class("key fn")`.

### Keys

`with_key` is how reconciliation keeps identity across reorders. Without one,
siblings match by position — fine for a static list, wrong for a list that
reorders, where the retained state (focus, an editing session, a running
animation) follows the position rather than the item.

Duplicate keys under one parent are recorded and reported rather than silently
resolved; see `ReconciliationPlan::duplicate_keys`.

## Properties

```rust
ViewProperty::new(name: InternedSymbol, value: ViewValue)
```

`ViewValue` is `Null | Bool(bool) | Int(i64) | Float(f64) | Text(Box<str>)`.

Reserved property names, all in `obscur_kit::view::view`:

| Symbol | Meaning |
|---|---|
| `CLASS` | CSS classes, space-separated |
| `ID` | CSS `#id` |
| `LOCAL_NAME` | the element's own name — what a CSS *type* selector matches: `button`, `input`, `textarea`, … |
| `TYPE` | the authored `type="…"` attribute — what `[type=submit]` matches |
| `SRC` | image or vector source |
| `DISABLED`, `CHECKED`, `READONLY` | control state, matched by `:disabled` / `:checked` |
| `VALUE` | a control's value |
| `PLACEHOLDER` | a field's placeholder |
| `TEST_ID`, `AUTOMATION_ID` | stable selectors for tests and automation |

`LOCAL_NAME` is what makes the UA stylesheet apply: a node named `button` picks
up the UA button rule, padding and border included. See
[04 — Styling](04-styling.md).

A view built by hand sets `LOCAL_NAME` explicitly; markup does not have to —
every authored element tag becomes its own `LOCAL_NAME`, so `<td>` is typed `td`.
`TYPE` is a different channel: it carries only what the author wrote as
`type="…"`, so `<button type="submit">` is a `button` that also answers to
`[type=submit]`. See
[09 — Authoring formats](09-authoring-formats.md#the-tag-and-type-are-two-channels-and-two-symbols).

## Events

```rust
ViewEventBinding::new(event: InternedSymbol, command: InternedSymbol)
```

A binding names an event and the **command** it invokes. Events flow down; the
command flows up to whatever handler the routing chain finds — the view never
holds a closure.

Reserved event names:

| Symbol | Fires |
|---|---|
| `EVENT_CLICK` | a click (or tap) resolves on the node |
| `EVENT_INPUT` | per keystroke, on an editable |
| `EVENT_CHANGE` | once, when an editable commits |
| `EVENT_LONG_PRESS` | a held press, after 0.5 s |

**Binding `EVENT_INPUT` is what makes a node a text field.** The runtime has no
other notion of editability: the editing session, caret rendering, IME routing
and [`focus_accepts_text`](03-runtime.md#focus) all key off it.

Handlers are registered on the runtime, not the view — `on_command`,
`on_command_at`, `on_scope_command`, `add_event_listener`, `on_text_input`,
`on_change` — or on `App`, which now exposes the first three. A command handler
takes nothing or takes the `&mut UiEvent` §2.3 dispatches; see
[03 — The runtime](03-runtime.md#commands).

## Virtual lists

Two mechanisms with different owners.

**`ViewNode::virtual_list(id, count, row_extent)`** — the runtime owns the
windowing. It builds only the rows that fit through the builder registered under
`id`, sizes the container from `count × row_extent` so the scrollbar is
proportioned honestly, and maps clicks back to model indices.

```rust
app.virtual_list(TAPE, move |index| row(index))
   .on_row_click(TAPE, move |index, _clicks| select(index))
```

`with_virtual_scroll_reset(token)` — change the token when the content is
*replaced* rather than appended, and the list returns to the top instead of
holding a scroll offset into different data.

**`ViewNode::virtual_collection(row_extent, template)`** — the template form, and
what a markup `<VirtualFor>` emits on both front ends.

> **partial** — the emitted template carries **no item binding**, so every
> realized row is identical. For rows that differ per item, use the
> runtime-registered form above: it is told each row's model index.

Reserved symbols behind both: `VIRTUAL_LIST_ID`, `VIRTUAL_ITEM_COUNT`,
`VIRTUAL_ROW_EXTENT`, `VIRTUAL_SCROLL_RESET`, `VIRTUAL_COLLECTION`,
`VIRTUAL_SPACER`, with `DEFAULT_VIRTUAL_ROW_EXTENT = 24.0`.

## Slots

`ViewNode::slot(name)` marks a projection outlet; a component spells
`<slot name="…"/>` in its own render and the runtime projects authored children
into it. A stray outlet — one with nothing to project — is refused by the
reconciler and reported.

`ViewNode::slot("")` — an outlet naming **no** slot — is the *default* one: it
takes the authored content that names none, text runs included, which is the
only way a `<button>Save</button>` can reach a mounted control at all. An outlet
nothing fills renders its own children as **fallback** content, which is how one
outlet serves both `<button>Save</button>` and `<button label="Save"/>`. The
shipped `button` is the one component that uses either, and its contract
declares the slot under the reserved `DEFAULT_SLOT` name — spelled
`#[contract(slot(default))]` on a derived contract.

A slot name is checked against the contract, from both sides: a component whose
render holds an outlet for a slot it does not declare, and an authored child
asking for a slot the component does not declare, are each reported as
`view.undeclared_slot`. The contract is what makes this checkable, because it is
static — a slot the author left empty is not a mismatch, and neither is an outlet
rendered only under some condition.

> **partial** — the default slot and fallback content exist in the runtime only:
> `.okui` refuses `<slot/>` and refuses children on an outlet, because neither
> has an authored spelling yet. Both diagnostics say that rather than saying the
> feature does not exist.

## Building views in practice

You will rarely spell `ViewNode::element(ComponentId::from_raw(…))` by hand.
[05 — Components](05-components.md) covers `Container`, `Text`, `Image`,
`Scroll` and the widget tier, which are thin builders over exactly these calls
and return `ViewNode` through `into_view_node()`.
