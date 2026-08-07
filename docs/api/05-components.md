# 05 — Components

Two tiers of thin builders. **They do not share a terminal method**, and that is
the first thing to get right:

| Tier | Ends with |
|---|---|
| Primitives | `into_view_node() -> ViewNode`, or `view() -> View` for a root |
| Widgets | `view() -> View` — **only `Button` also has `into_view_node()`** |

Reaching for `into_view_node()` on an `Input`, a `Slider` or a `Select` is
`error[E0599]: no method named into_view_node`. A widget nests into a bigger tree
through `View::into_single_node() -> Option<ViewNode>`, or by reading its
`root() -> &ViewFragment`; `Button` is the exception because it is the one widget
a caller most often nests directly.

- **Primitives** (`obscur_kit::primitives`) — structure and content, no look and
  no behaviour of their own.
- **Widgets** (`obscur_kit::widgets`) — controls, each binding a command, eight of
  them carrying a CSS *type*.

Both are headless: they produce view nodes and nothing else. There is no widget
state hiding behind them — a checkbox is checked because you said `.checked(true)`.

---

## Primitives

Every one ends with `into_view_node() -> ViewNode`, or `view() -> View` when it
is the root.

| Type | Constructor | Builders |
|---|---|---|
| `Container` | `Container::new()` (also `Default`) | `child(ViewNode)`, `with_children(impl IntoIterator<Item = ViewNode>)`, `len()`, `is_empty()` |
| `Text` | `Text::new(impl Into<Box<str>>)` | `content()` |
| `Image` | `Image::new(impl Into<Box<str>>)` | `with_alt(impl Into<Box<str>>)` |
| `Scroll` | `Scroll::new()` | `child(ViewNode)` |
| `Layer` | `Layer::new()` | `child(ViewNode)`, `with_opacity(f32)` |
| `Portal` | `Portal::new(target: impl Into<Box<str>>)` | `child(ViewNode)` |
| `Canvas` | `Canvas::new(width: f32, height: f32)` | `named(InternedSymbol)` — the painter this canvas binds itself |
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

Every one of these ends in `view() -> View`. Where a widget has a CSS type, that
is the name a type selector matches — and, for the six the
[UA stylesheet](04-styling.md) has rules for, what your own sheet must override if
you restyle it.

| Widget | Constructor | Builders | CSS type |
|---|---|---|---|
| `Button` | `new(label, command)` | `disabled(bool)`, `label()` | `button` |
| `Checkbox` | `new(label, command)` | `checked(bool)`, `disabled(bool)` | `checkbox` |
| `Radio` | `new(label, command)` | `selected(bool)`, `disabled(bool)` | `radio` |
| `Input` | `new(command)`, `with_value(command, value)` | `placeholder(…)`, `multiline(bool)`, `disabled(bool)` | `input`, or `textarea` when multiline |
| `Select` | `new(command)` | `option(label)`, `with_options(iter)`, `disabled(bool)`, `open()`, `selected_label()` | `select` |
| `List` | `new()` | `item(label)`, `with_items(iter)`, `selected_indices()` | — |
| `Table` | `new()` | `column(Column)`, `with_columns(iter)`, `row(cells)`, `with_rows(iter)`, `row_count()`, `column_count()` | `table`, and its parts `tr` / `th` / `td` |
| `Tabs` | `new()` | `tab(label)`, `with_tabs(iter)` | — |
| `Tree` | — | built from `TreeNode::new(label, depth, has_children)` | — |
| `Menu` | `new()` | `item(MenuItem)`, `with_items(iter)`, `open()` | — |
| `MenuItem` | `new(label, command)` | `disabled()` | — |
| `Dialog` | `new(title)` | `child(ViewNode)`, `open()`, `title()` | — |
| `Docking` | `new()` | `with_panels(iter)`; panels are `DockPanel::new(id, title, DockSide)` | — |
| `Column` | `new(title)` | (a `Table` column) | — |
| `Slider` | `new(value)`, `ranging(low, high)` | `over(min, max)`, `stepped_by(f32)`, `value()`, `range()`, `scale()` | `slider` |
| `Progress` | `new(value)`, `indeterminate()` | `shaped(ProgressShape)`, `along(path_data)`, `value()`, `is_tracked()` | `progress` |

**Eight widgets carry a CSS type**, and the set is no longer the same as the set
the UA stylesheet styles. Six of them — `button`, `input` / `textarea`,
`checkbox`, `radio`, `select`, `table` (with `tr` / `th` / `td`) — have UA rules.
The other two do **not**:

> `Slider` and `Progress` each emit a `LOCAL_NAME` of `slider` / `progress`, so
> `slider { … }` and `progress { … }` match them from your own sheet — but the UA
> stylesheet has **no rule for either**, and it is not an oversight. Both draw
> themselves through a registered canvas painter rather than through box
> decoration, so a UA rule would have nothing to say that the painter does not
> already decide. Change their look with `App::painter_id(Slider::PAINTER, …)` /
> `(Progress::PAINTER, …)`, which is applied after the bootstrap's and therefore
> wins. For the colour alone there is
> `BuiltinPainters::new().with_accent(c).install(&mut runtime)` — but note that
> `KitBootstrap::install_builtin_painters` always installs the *default* accent,
> so reaching that constructor means a `control().with_runtime(…)` or a runtime
> you own. `tests/ua_stylesheet_vocabulary.rs` checks that every rule in the sheet
> reaches something buildable — it does not check the converse, which is why this
> asymmetry is legal.

The widgets with no CSS type at all set a private `role` property instead — that
is not a class and not a local name, so a CSS type selector cannot match it. To
style a `List` or a `Menu`, use classes.

`Table` is the one that changed: its rows and cells are real element nodes, and
they used to carry that private `role` (the text `row` / `cell`) rather than a
name. The sheet's `table`, `th`, `td` and `tr:hover` rules were therefore dead
against every table a program built while matching hand-written `<table>` markup
perfectly. The parts name themselves now — `table`, `tr`, `th`, `td` — so one
sheet styles both routes, and the accessibility projection announces a table
rather than a stack of unnamed groups.

`img` was the same defect one tier down, and is fixed the same way. `<img>` in
markup and `Image::new(…)` in Rust are **one built-in** —
`name_binding::builtin_component_id("img")` *is* `Image::COMPONENT_ID` — but the
tag contributed a local name and the primitive contributed none, so one `img`
rule reached one of the two trees, and the accessibility projection announced a
primitive image as an unnamed group. The primitive names itself now.

The sheet's remaining hand-written vocabulary is `a` and `fieldset`: **nothing
in the widget or primitive tier sets those types**. They match a node an
application names by hand with the `LOCAL_NAME` property — and, since
`name_binding::local_name_for`, every element tag written in markup: an authored
`<a>` is typed `a` by its tag alone, so those rules reach `.okui` / `okui!`
markup without an application naming anything. Those two are also the whole of
the escape hatch in `tests/ua_stylesheet_vocabulary.rs`, the guard that now
rejects a UA rule for anything the toolkit cannot build.

### Five of them mount from markup

`Checkbox`, `Radio`, `Select`, `Input` and `Button` are not only builders: each
ships a props struct, an `impl Component` and a `ComponentFactory`, and
`KitBootstrap` registers all five by default. So an authored tag is the real
control, not a styled box with the right id:

```html
<checkbox label="Agree" checked/>
<radio label="One" checked/>
<select options="Alpha, Beta" selected={1} open/>
<input value="abc" placeholder="Search"/>
<textarea value="line one" on:input={edit}/>
<button on:click=save>Save</button>
```

Five things about that markup are worth stating rather than inferring.

**A toggle's authored state is `checked`, the reserved attribute** — for the
radio too, where the *builder* method is `selected(bool)`. `checked` is what the
cascade reads for `:checked`, so a control cannot paint a filled dot while
`radio:checked` fails to match it. Presence is the statement: a bare `checked`
means checked.

**A select's options are a prop, not children.** `Select::with_options` takes
labels, not views, and the control synthesises each `.select-option` row around
one — so `options="Alpha, Beta"` is a comma-separated list, `selected` is an
index and `open` is present-or-absent. The consequence, stated plainly: **an
option label containing a comma cannot be authored in markup.** It can be built
in Rust and it survives there, because `Select::view` emits its own list under a
separator no typed label contains.

**A field's text is `value`, and `<input>` and `<textarea>` are one control.**
They share `Input::COMPONENT_ID` and one registration; which of them a node is
rides on its *local name*, which is what decides multi-line editing — so an
authored textarea's `Enter` inserts a newline and an input's does not, exactly
as before. The `placeholder` is not the component's output at all: §4.16 makes
it a synthetic part the runtime paints from the reserved attribute while the
field is empty, style it with `input::part(placeholder)`. Since this release an
authored `placeholder="…"` reaches it; before, only `Input::placeholder(…)` did,
and the authored hint painted in full ink or not at all.

**Four of these controls own their content.** A component's output replaces its
host's children, so authored content under `<checkbox>`, `<radio>`, `<select>`,
`<input>` or `<textarea>` could not render. That is a compile error rather than
a silent loss:

```html
<textarea>Hello</textarea>   <!-- rejected: use value="Hello" -->
```

HTML makes a textarea's child text its default value; here that is spelled
`value="…"`. No policy applies to `<button>` or `<table>`, and their children
still render: `<table>` mounts nothing at all, and what mounts on `<button>`
owns no children — see below.

**A button's children are yours.** `<button>` is the one mounted control whose
content is the author's, so it is the one that declares a §2.12 **slot** — the
default one. Its component renders that outlet and nothing else, and the runtime
projects your children into it before the frame is reconciled:

```html
<button on:click=save>Save</button>          <!-- your text -->
<button on:click=save><b>Save</b> now</button>  <!-- your markup, all of it -->
<button on:click=save label="Save"/>         <!-- the label is the fallback -->
```

The third line is the one that changed: `label="…"` is what the `.okui`
compiler requires of `<Button label="…"/>`, and until this release it rendered
an empty button. It is the outlet's *fallback* now — shown only when you supply
no content, so writing both never loses your markup.

Registering a built-in does not touch the host node. Your `class`, your
`on:change`, your `disabled` and the `:checked` / `:disabled` the cascade reads
are all still the authored node's; only what renders underneath is the
control's. And a widget you built in Rust renders exactly what it always did —
`Checkbox::new(…).view()` carries the same id, so it mounts too, and the
component regenerates the same children the builder installed.

`<table>` is the one lowercase control tag still mounting nothing, and that is
a decision rather than a gap: its rows and cells are `tr`/`td`/`th` elements you
write or `Table::view()` builds from data, so a component there could only hand
them straight back. It stays an author-owned base element.

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
    .view()                       // -> View. `Input` has no `into_view_node`.
```

Register the handler on the runtime side with `on_text_input` (per keystroke) or
`on_change` (on commit).

## Themes

`obscur_kit::theme_system` holds the token tier — a theme is a set of named
values a stylesheet reads. See [04 — Styling](04-styling.md#themes).

## What is not here

- **No user-authored components *here*.** The tier this page documents is the
  built-ins. Authoring your own works — `#[derive(Contract)]`, `<Card/>` in either
  front end, `register_component` on the runtime, and §2.12 slot projection — but
  this repository ships no *user* component, so nothing below is one. The five
  built-ins that are components (above) are not an exception to that: they are
  registered under reserved ids a base-element tag already binds to. See
  [09 — Authoring formats](09-authoring-formats.md#user-components-and-slots) for
  what is and is not checked about the tag.
- **No context menus.** §6.7 requires custom context menus; `Menu` is a menu
  surface, not a context menu.
- **Docking is structure only** — panels are described and laid out; dragging one
  to re-dock it is not built.
