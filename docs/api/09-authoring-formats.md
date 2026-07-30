# 09 — Authoring formats

Three authoring surfaces, four routes — a `.okui` file has two, and they are not
equivalent. Pick the route before you pick the markup: what a `{binding}` can
read and when a contract error reaches you both follow from it.

| Route | Turned into Rust by | A `{binding}` in a body reads | Contract errors |
|---|---|---|---|
| Rust builders | rustc | anything in scope | there is no markup to check |
| `okui! { … }` | rustc, during expansion | the caller's locals | a compile error |
| `.okui` + `build.rs` | your build script | a `const`/`static`/import only | a build-script failure |
| `.okui` + `App::screen` | nothing — the UI IR is kept and re-lowered per frame | a `ScreenData` entry | an `AppError` from `run` |

The first three leave **no interpreter at runtime** — no IR, no `.okui` machinery
and no document survives compilation. `App::screen` is the exception: it keeps
the compiled IR for the life of the process and lowers it to a `View` on every
frame. It does not re-parse and does not re-compile.

---

## The `okui!` macro

`obscur_kit::ui_macros::okui!` parses a TSX-like tree, lowers it to the UI IR and
runs it through `ViewCodegen` **during expansion**, so what lands in your crate
is plain Rust.

```rust
let label = format!("count: {count}");
let view = okui! {
    <div class="app">
        <div class="count"><span>{label}</span></div>
        <button class="btn" on:click={increment}><span>"press"</span></button>
    </div>
};
```

Also exported: `#[derive(Contract)]`, which emits a component's
`contract_surface()` for §2.13 validation, and computes the component's id from
its name the same way an authored tag does.

**What your crate needs:** `obscur-kit`, and nothing else. Every path in the
expansion is absolute and written through the facade
(`::obscur_kit::view::view::…`, `::obscur_kit::identity::…`), so no `use` item and
no second dependency is required. Until `191d79e` this was untrue: the expansion
named `::obscur_view` and `::obscur_identity`, and a `::`-prefixed path resolves
in the **extern prelude** — a crate's direct dependencies only — so the first
`okui!` in an application failed with `error[E0433]: cannot find obscur_view in
the crate root`. No re-export could have fixed that; the root had to become an
argument, which is `CodegenRoot`.

`checks/facade-consumer` is a crate outside the workspace whose only dependency
is the facade; `cargo run -p xtask -- consumer` builds and tests it. It exists
because no in-repo consumer stands on that side of the boundary, which is why the
defect above survived as long as it did.

## A `.okui` file at build time

The route §2.9 describes when it asks for compile-time contract validation. A
build script parses the file, compiles it, and writes ordinary Rust the crate
`include!`s:

```rust
// build.rs
let result = Compiler::new("screen.okui")
    .claiming_command("increment")          // §2.13: handler names are checked
    .compile(&document);
assert!(!result.has_errors());              // lowering does not stop at a diagnostic
let rust = ViewCodegen::document_to_view_fn(
    result.document().unwrap(), "screen_view", &CodegenRoot::facade());
```

`CodegenRoot::facade()` is the argument an application must pass;
`CodegenRoot::internal()` is for a crate that depends on `obscur-view` and
`obscur-identity` directly, which an application does not.
`crates/facade/obscur-kit/build.rs` is the worked example in this repository.

Two limits worth knowing before you commit to it:

- `document_to_view_fn` emits a **nullary** `pub fn <name>() -> View`, so a
  binding has no per-call state to resolve against: only a `const`, a `static` or
  an import is in scope. Giving it parameters needs a decision about the props
  signature that nothing here makes yet.
- a `build.rs` cannot register a contract for a component defined in **its own**
  crate — `[build-dependencies] X` inside `X` is a cargo `cyclic package
  dependency`. A component from another crate is fine.

This is also the only route that can check handler names: pass every command the
application registers to `Compiler::claiming_command`, or none.

## A `.okui` file at run time

```rust
App::new("Showcase").screen(include_str!("showcase.okui")).run()
```

`App::screen` parses and compiles the markup inside `run` — before any window
opens, so a failure is `AppError::ScreenParse` / `AppError::ScreenCompile` with
one message per diagnostic rather than a blank window — then lowers the kept IR
to a `View` every frame through `OkuiScreen`.

```html
<app id="app" class="app">
  <header class="header">
    <span class="title">{title}</span>
  </header>
  <div class="gallery">
    <div type="card" class="solid">…</div>
  </div>
</app>
```

**Values reach it; shape does not.** A body `{count}` and a bound attribute
(`class={theme}`) read the `ScreenData` table `App::screen_data()` hands out, so
writing that table between frames changes what the screen shows. `<If>` and
`<For>` are materialised statically by the bridge on this route, so the
*structure* is fixed once and for all.

A name an **attribute** binds must be in the table *before* `run`: that is what
declares it to the compiler, and §2.13 makes a reference to anything else an
error from `run` rather than a value that silently never arrives. Body
interpolations are not name-checked on this route — an unbound `{count}` renders
its own path, visibly.

Until recently this materialised the screen **once** and handed the runtime a
builder returning that one `View` for the process's whole life, so no state change
could reach a `.okui` file at all — and this page said only "compiles the markup at
`run`", leaving the consequence to a comment on one of 53 examples. Holding the IR
instead of the `View` costs one lowering per frame and gives that lowering
somewhere to read state from.

## `on:click={name}` — a name, not an expression

Inside `okui!`, `{ident}` means **two different things by position**, and nothing
in the syntax says so:

- in **body** position it is a Rust binding from the enclosing scope, emitted
  verbatim and resolved by rustc;
- in an **`on:` attribute** it is a *command name*. It is hashed
  (`name_binding::fallback_symbol`, equal to `InternedSymbol::named` by a drift
  test) into the command symbol the runtime dispatches on. Nothing evaluates it,
  and there need be no Rust binding of that name anywhere.

`checks/facade-consumer/src/counter.rs` has both in one function: `{label}` is a
local, while `increment` appears only in the markup and in
`InternedSymbol::named("increment")`. `examples/file_manager.rs` is the same
shape at scale — `{path}` is a local, `goParent` is a name.

The file front end's name resolver skips `on:` values for exactly this reason:
resolving one against the props/state table asks the wrong question, and used to
report `on:click={increment}` as an unknown *name* whose only remedy was to declare
a state field that has nothing to do with it.

Which spellings each front end accepts:

| Spelling | `okui!` | `.okui` file |
|---|---|---|
| `on:click={increment}` | **the only form** | accepted |
| `on:click=increment` | a parse error — the value must be braced | accepted |
| `on:click="increment"` | a parse error | parses, then refused as string dispatch on any tag with a registered contract (§2.3: "String-based method dispatch is not used") |

The quoted form is the one to avoid: it is refused where a contract exists and
silently accepted where none does (a plain `<div on:click="save"/>`), which is the
worst of the three outcomes.

Handler *names* are only checked on the build-time route, through
`Compiler::claiming_command`. Everywhere else a misspelled name hashes cleanly to
a symbol nobody registered, and the button does nothing.

## Attributes

Nine attribute names are **reserved** — they bind to the `obscur-view` symbols
the reconciler, the CSS matcher and the accessibility projection read
(`name_binding::attr_symbol`). Every other name is hashed into an ordinary prop:
unlisted is not rejected, it is just inert to those consumers.

| Attribute | Binds to | Read by |
|---|---|---|
| `class` | `CLASS` | `.class` selectors |
| `id` | `ID` | `#id` selectors |
| `type` | `TYPE` | type selectors — see below |
| `src` | `SRC` | the image/vector resource path |
| `disabled` | `DISABLED` | `:disabled` |
| `checked` | `CHECKED` | `:checked` |
| `test-id` | `TEST_ID` | §7.5 test targeting; `SelectorTarget::test_id` |
| `automation-id` | `AUTOMATION_ID` | §7.5 external automation |
| `slot` | `SLOT` | §2.12 projection — which outlet this child fills |

One binding depends on the tag it sits on: `name` on a `<slot>` is the outlet's
slot name and binds to the same `SLOT` symbol, because the outlet and the child's
assignment are two halves of one join. On any other tag `name` is an ordinary
hashed prop. (`name_binding::attr_symbol_on` is that rule; `attr_symbol` is the
tag-blind entry point the `Contract` derive uses, where there is no tag.)

Two more are structural rather than properties: `key` (a `<For>` child's identity
— carried on the loop node itself, not duplicated as a prop) and `requires="cap"`
(the §2.13 capability declaration). Both are always permitted by a closed
contract.

### What an attribute may be called

Both front ends accept a hyphenated name, and they accept it in the same shape:
one or more identifier segments joined by `-`. This is not a detail — two of the
nine reserved names above are hyphenated, and they are the two an application
*must* be able to write, because `App::on_command_at` and
`App::add_event_listener` name their target with `SelectorTarget::test_id` /
`::automation_id`, an identifier that has to exist before the tree does.

Until the fix for audit finding 2-3, `test-id="a"` was a **parse error** in
`okui!` and accepted in a `.okui` file. `okui!` is a proc-macro over Rust tokens,
where `-` is punctuation, and its parser had no hyphen handling; the file front
end has its own lexer whose identifier rule admits `-`. So §7.5's identifiers —
and `data-*`, and any future hyphenated reserved name — were authorable on the
*fallback* route only, on a page that calls the macro primary.

| Spelling | `okui!` | `.okui` file |
|---|---|---|
| `class="row"` | accepted | accepted |
| `test-id="a"`, `automation-id="a"`, `data-row-index="3"` | accepted — joined in name position | accepted |
| `on:pointer-down={go}` | accepted — the event name joins the same way | accepted |
| `test-` (a trailing hyphen) | an error naming the hyphen | accepted by the lexer as an identifier |
| `data-1` (a segment starting with a digit) | a parse error — Rust hands the macro a *literal*, not an identifier | accepted |
| `a.b` (a dot inside a name) | a parse error | accepted |
| `<my-tag/>` (a hyphenated **tag**) | a parse error | accepted |

The last four rows are the exact width of the disagreement that remains, and it
is one-directional: the file lexer is looser. Its identifier rule is shared with
*value* position, where `value=item.name` has to lex as one token, so the dot and
the digit-initial segment are admitted there incidentally rather than designed;
none of them spells a reserved attribute, and none is reachable through Rust's
tokenizer without inventing a syntax the file front end does not have. The
hyphenated tag is admitted for the same lexical reason and means nothing on
either front end — it names no primitive, no widget and no component, so it
hashes to an id nothing registers and mounts an empty box.
`crates/facade/obscur-kit/tests/okui_macro.rs` measures all four rather than
inferring them.

Joining happens only in **name** position — after `<Tag`, between attributes, and
after `on:`. Nowhere else could it: a `-` inside `{ … }` is subtraction, and a
brace group reaches `syn::Expr` whole, so `<div title={count - 1}/>` means what it
says.

### `type` is two channels through one symbol

Since `445b6bd` every authored **element** carries its tag as the reserved `TYPE`
property — its CSS local name — so `button { … }` matches an authored `<button>`
exactly as in HTML. `TYPE` is also what an authored `type="…"` binds to. There is
one slot and two writers, and the **author wins**:

- `<div type="card">` works as before: the node's CSS type is `card`, which is
  what the sample above styles.
- `<button type="submit">` costs the node its `button` type. It is an element
  whose CSS type is `submit`, so the UA sheet's `button { … }` no longer reaches
  it and neither does yours.

If you want both, say so with a class. This is the price of the overload; it is
not papered over.

## Known tags

### Base elements

Any lowercase tag is an element: it lowers to a real, styleable box and carries
its own tag as its CSS type. A few lowercase spellings additionally name a
built-in, and are the same handle as their PascalCase form:

| Element | Is |
|---|---|
| `button` | the `Button` widget's id — HTML has a `<button>`, so this has one |
| `img` · `canvas` · `scroll` · `portal` · `layer` | the matching primitive |
| `slot` | the §2.12 outlet. Reserved: not a component, and never retained |

Every other lowercase tag — `div`, `span`, `header`, `td`, `something-custom` —
is the generic `Container` box with its tag as its type.

`button` is the only *control* with a base-element spelling. `<input>`,
`<textarea>`, `<checkbox>`, `<radio>` and `<select>` bind to `Container` — but
they pick up the UA stylesheet's rules for those types, and the accessibility
projection derives a role from the local name, so they look and read like
controls without being one. That asymmetry is worth knowing: styling and
semantics follow the *name*, mounting follows the *id*.

An authored `<button>` is not a live `obscur_widgets::Button` either. Nothing
registers a factory for the built-in widget ids by default, so the widget's own
view — its pressed state, its synthesised label — does not run. What you get is a
node with the widget's id, the `button` type, the UA sheet's padding and border,
your children, and your `on:click`.

### Component tags

| | |
|---|---|
| Widgets (twelve) | `Button`, `Input`, `Checkbox`, `Radio`, `Select`, `Dialog`, `List`, `Menu`, `Tabs`, `Tree`, `Table`, `Docking` |
| Primitives (six) | `Image`, `Canvas`, `Scroll`, `Portal`, `Layer`, `ForeignSurface` |
| Anything else | a **user component**: the id is `user_component_id(tag)`, the same id `#[derive(Contract)]` computes for that name, so a host's `register_component` and an author's `<Card/>` meet on the string |

`Container` and `Text` are **not** in that list, although both have a contract
registered under their name. Neither names a component id: `Container`'s id is what
an *element* tag falls back to, and `Text` has none at all (the primitive is a text
node). So `<Container/>` and `<Text/>` are user-component tags nobody registered —
anonymous boxes. Write a lowercase tag for a box, and bare text or `{expr}` for
content.

### Control flow

`<If>` · `<ElseIf>` · `<Else>` · `<For>` · `<VirtualFor>`, plus `<Empty>` and
`<Separator>` as a loop's §2.11 slots. `<Loading>` is deliberately absent — no
signal defines when it would show.

On the compiled routes these are real Rust: `<If cond={open}>` emits `if open`,
and `<For each={items} as={item} key={item.id}>` emits `for item in &items`, so
the body's `{item.name}` is an ordinary path. §2.11 makes a `<For>`'s `key`
mandatory, and a keyless one is a diagnostic on the file path only — `okui!` lowers
it silently and every row renders unkeyed.

`<VirtualFor>` emits **no** loop. It is a §4.9 performance contract, so it lowers
to a single virtual-collection marker carrying its row template, which the
runtime realizes into a viewport-bounded window. Its template is emitted **once
with no item binding**, so every realized row is identical: for rows that differ
per item, use `App::virtual_list` / `UiRuntime::register_virtual_list`, which is
told each row's model index.

`<Slot>` is not a control-flow tag and never was. The outlet is **lowercase
only** (`name_binding::OUTLET_TAG` is `"slot"`), because both front ends classify
a lowercase tag as an element and giving the outlet two spellings would mean that
classification decided nothing about it. A `<Slot/>` is what any unrecognised
component tag is — `user_component_id("Slot")`, mounting nothing, with no
diagnostic.

### The typo that stays silent

That is the general rule, and it is the one gap in this section: **nothing checks
a component name.** `<Buton/>` hashes to its own id, nothing is registered under
it, and it renders as an anonymous box. §2.13 closes typos in the *attributes* of
a known component, and `Compiler::claiming_command` closes typos in handler
*names* on the build path. A misspelled tag is closed by neither, and cannot be
by the macro: the registry it would consult is a runtime value of
`Box<dyn ComponentFactory>`, and the macro runs while that crate is still being
compiled.

## Validation

Both front ends run the same rules from the same registry
(`ContractRegistry::builtins()`, the primitives plus all twelve widgets), stated
over plain data so the macro can reach the verdict from `syn` tokens and the file
path from its syntax tree. Each attaches its own location — a `Span` for the
macro, a byte offset for the file.

| Rule | Spec | What it catches |
|---|---|---|
| required props | §2.13 | `<Button on:click={go}/>` — no `label` |
| unknown attributes | §2.13 | `<Button laebl="OK"/>`, once the contract declares any prop |
| unknown events | §2.13 | `<Button on:hover={go}/>`, once it declares any event |
| allowed children | §2.13 | see below |
| slots | §2.12 | a `slot="…"` naming nothing, a slot's own child rule, and cardinality — reported only when it is *certainly* wrong, so an `<If>`-guarded child is never accused |
| the outlet | §2.12 | `<slot>` with no `name`, a `name={expr}`, any other attribute, or children |
| expressions | §2.10 | assignment, mutation, I/O, `async`, forbidden keywords — in an attribute **and** in a body `{expr}` |
| capabilities | §2.13 | a `requires="cap"` the caller did not grant |
| string dispatch | §2.3 | `on:click="save"` on a tag with a contract |
| handler names | §2.13 | file path only, and only when the caller declares the surface |

**Allowed children** is a three-state policy, not a list whose emptiness had to
mean two things. `Any`, `Only([tags])`, or `None` — and `None` is the state
almost every shipped component wanted: eleven of the twelve widgets build their
children inside their own `view()`, and `Image`/`Canvas`/`ForeignSurface` are
leaves. So `<Button>OK</Button>` and `<Tabs><Tab/></Tabs>` are compile errors
naming the tag, where they used to compile into markup nothing mounts. `Dialog`
is the one widget whose children *are* its content.

That is a claim about the **component tag**, not about every spelling of the same
id: contracts are keyed by the tag the author wrote, so `<button>` keeps its
children — they are the only thing that renders for a base element — and so does
`<img>`.

### The five that cannot be filled from markup

`Menu`, `Tabs`, `Tree`, `Table` and `Docking` resolve to real ids and carry real
contracts. What they do not have is a *string* content vocabulary: their content
is typed Rust values — `MenuItem`, `TreeNode`, `Column`, `DockPanel` — with no
spelling an attribute or a child could name. Their prop surface is therefore left
deliberately **open** rather than asserting a vocabulary that does not exist.

The practical effect: `<Tabs/>` and `<Tabs anything="1"/>` compile clean, and only
a child (`<Tabs><Tab/></Tabs>`) is refused. What reaches the tree is a node with
the widget's id and nothing in it — an empty box, because nothing registers a
factory for a built-in widget id by default. That is worse than the compile error
a reader expects there, and it is why "reachable from markup" is not the same claim
as "usable from markup". Build these from Rust —
[05 — Components](05-components.md) — and inject the result where you need it.

The day a `<Tab>`/`<Column>` vocabulary is specified, each policy becomes
`Only([…])`. Until then this paragraph is the honest version of what the page
used to call "no markup contract".

## User components and slots

A registered user component renders from either front end: the tag's id is
derived from its name, `#[derive(Contract)]` derives the same id from the same
name, and `register_component` mounts the factory under it. `<Card><h1
slot="header"/></Card>` reaches the `ViewNode::slot("header")` the component's
render declares — projection is real, and a component that declares no outlet
drops the children it was given, which is §2.12 working rather than missing.

What is *not* closed: an unregistered tag is silent (above), and a component's own
`<slot name="hedaer"/>` is not checked against its declaration — it simply
receives nothing. No shipped component declares a slot, because no shipped widget
is a `Component`; slots are toolkit API for a host's own components.

## The UI IR

`obscur_kit::ui_ir` is the shared middle: every front end lowers into it,
`UiIrViewBridge` interprets it at runtime and `ViewCodegen` lowers it to Rust
ahead of time. Its `name_binding` table is the single policy behind everything on
this page — tag → `ComponentId`, tag → local name, attribute → symbol, event →
symbol — and both lowering paths call it rather than open-coding a rule, because
the bytes they emit have to stay identical. A drift test in the facade pins every
constant in it against its canonical definition in `obscur-view`,
`obscur-primitives` and `obscur-widgets`.

You would touch the IR only to write a third front end.
