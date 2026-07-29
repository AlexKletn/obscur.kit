# 05 — Components

Two tiers, both thin builders that end in `into_view_node()`.

- **Primitives** (`obscur_kit::primitives`) — structure and content, no look and
  no behaviour of their own.
- **Widgets** (`obscur_kit::widgets`) — controls, each binding a command, five of
  them carrying a CSS *type* the UA stylesheet matches.

Both are headless: they produce `ViewNode`s and nothing else. There is no widget
state hiding behind them — a checkbox is checked because you said `.checked(true)`.

---

## Primitives

Every one ends with `into_view_node() -> ViewNode`, or `view() -> View` when it
is the root.

| Type | Constructor | Builders |
|---|---|---|
| `Container` | *(Default)* | `child(ViewNode)`, `with_children(impl IntoIterator<Item = ViewNode>)`, `len()`, `is_empty()` |
| `Text` | `Text::new(impl Into<Box<str>>)` | `content()` |
| `Image` | `Image::new(impl Into<Box<str>>)` | `with_alt(impl Into<Box<str>>)` |
| `Scroll` | `Scroll::new()` | `child(ViewNode)` |
| `Layer` | `Layer::new()` | `child(ViewNode)`, `with_opacity(f32)` |
| `Portal` | `Portal::new(target: impl Into<Box<str>>)` | `child(ViewNode)` |
| `Canvas` | *(Default)* | — |
| `ForeignSurface` | `ForeignSurface::new(impl Into<Box<str>>)` | `surface()` |

`Container` is the workhorse — it is a bare element, and everything about how it
looks comes from its classes:

```rust
Container::new()
    .child(Text::new("Total").into_view_node().class("label"))
    .child(Text::new(&total).into_view_node().class("value"))
    .into_view_node()
    .class("row")
```

## Widgets

Where a widget has a CSS type, that is what the [UA stylesheet](04-styling.md)
matches — and what your own sheet must override if you restyle it.

| Widget | Constructor | Builders | CSS type |
|---|---|---|---|
| `Button` | `new(label, command)` | `disabled(bool)`, `label()` | `button` |
| `Checkbox` | `new(label, command)` | `checked(bool)`, `disabled(bool)` | `checkbox` |
| `Radio` | `new(label, command)` | `selected(bool)`, `disabled(bool)` | `radio` |
| `Input` | `new(command)`, `with_value(command, value)` | `placeholder(…)`, `multiline(bool)`, `disabled(bool)` | `input`, or `textarea` when multiline |
| `Select` | `new(command)` | `option(label)`, `with_options(iter)`, `disabled(bool)`, `open()`, `selected_label()` | `select` |
| `List` | `new()` | `item(label)`, `with_items(iter)`, `selected_indices()` | — |
| `Table` | `new()` | `column(Column)`, `with_columns(iter)`, `row(cells)`, `with_rows(iter)`, `row_count()`, `column_count()` | — |
| `Tabs` | `new()` | `tab(label)`, `with_tabs(iter)` | — |
| `Tree` | — | built from `TreeNode::new(label, depth, has_children)` | — |
| `Menu` | `new()` | `item(MenuItem)`, `with_items(iter)`, `open()` | — |
| `MenuItem` | `new(label, command)` | `disabled()` | — |
| `Dialog` | `new(title)` | `child(ViewNode)`, `open()`, `title()` | — |
| `Docking` | `new()` | `with_panels(iter)`; panels are `DockPanel::new(id, title, DockSide)` | — |
| `Column` | `new(title)` | (a `Table` column) | — |

**Only five widgets carry a CSS type**, and they are exactly the five the UA
stylesheet has rules for. The rest set a private `role` property instead — that
feeds the accessibility projection, and a CSS type selector cannot match it. To
style a `List` or a `Table`, use classes.

That cuts both ways: the UA sheet also carries `table`, `th`, `td`, `a`,
`fieldset` and `img` rules, and **nothing in the widget or primitive tier sets
those types**. They match a node an application types by hand with the `TYPE`
property — and, since `name_binding::local_name_for`, every element tag written in
markup: an authored `<td>` is typed `td` by its tag alone, so those rules reach
`.okui` / `okui!` markup without an application naming anything.

### The trap worth knowing before you style one

A widget node is a real control with the UA sheet's `padding: 11px 20px` and
`1.5px` border on it. Your class does **not** replace those — it cascades over
them, and anything you do not name stays. A keypad of `Button`s styled only with
`background-color` and `border-radius` keeps 40 px of horizontal padding, which
at small sizes collapses the content box and clips the label away entirely.

```css
.key { padding: 0; border: 0; /* … then your own look */ }
```

That is the browser contract — a `<button>` carries UA padding until an author
says otherwise — and it bites hardest on a small window at a display scale of 2,
where the *logical* viewport is half the pixels you see.

### Input

`Input` binds `EVENT_INPUT`, which is what makes it a text field to the runtime:
the editing session, caret drawing, IME routing and the on-screen keyboard all
follow from that binding rather than from the type name.

```rust
Input::with_value(InternedSymbol::named("name"), current)
    .placeholder("Your name")
    .into_view_node()
```

Register the handler on the runtime side with `on_text_input` (per keystroke) or
`on_change` (on commit).

## Themes

`obscur_kit::theme_system` holds the token tier — a theme is a set of named
values a stylesheet reads. See [04 — Styling](04-styling.md#themes).

## What is not here

- **No user-authored components.** §2.14 registration exists on the runtime
  (`register_component`), but no component can be authored in either front end
  yet, so the component tier is the built-ins.
- **No context menus.** §6.7 requires custom context menus; `Menu` is a menu
  surface, not a context menu.
- **Docking is structure only** — panels are described and laid out; dragging one
  to re-dock it is not built.
