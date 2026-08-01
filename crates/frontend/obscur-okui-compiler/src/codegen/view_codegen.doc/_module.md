`ViewCodegen`: emits ordinary Rust that builds a runtime [`View`] from a
compiled [`UiIrDocument`] at *compile* time (spec §2.8, §2.15).

This is the code-generation counterpart of the runtime
[`UiIrViewBridge`](crate::bridge::UiIrViewBridge): where the bridge
*interprets* the IR into a desired tree every time it runs, `ViewCodegen`
lowers the same IR once, ahead of time, into Rust source that constructs the
identical [`View`] with no `.okui` machinery present at runtime. A build step
writes the emitted function into a generated `.rs` file the application
`include!`s (or the same text is produced inline by the `okui!` macro).

# What it emits

[`document_to_view_fn`](ViewCodegen::document_to_view_fn) emits a free
function

```ignore
pub fn <name>() -> ::obscur_kit::view::view::View { <expr> }
```

whose body is the fully-qualified [`View`] expression from
[`document_to_view_expr`](ViewCodegen::document_to_view_expr). Every path is
written with a leading `::` so the generated code needs no `use` items, and
every one of them is written through the [`CodegenRoot`] the caller passes —
by default the `obscur-kit` facade, the one crate an application depends on.
See [`CodegenRoot`] for why that root is an argument rather than the fixed
`::obscur_view` / `::obscur_identity` literals it used to be.

# Name binding

Tags, attributes and events resolve through the shared
[`name_binding`] registry — the *same* policy the runtime bridge uses — so a
`<Button>` emits its real `COMPONENT_ID`, an `<img src=…>` emits the reserved
`SRC` symbol (reaching real decoded pixels), a `<div>` emits the generic
`Container`, and `click`/`input`/`change` emit the reserved `EVENT_*`
symbols. The emitted handles are raw values (`from_raw(0x…)`) resolved from
the registry at generation time.

An **element** tag is emitted twice over: as the component id above, and as the
reserved `TYPE` property carrying the tag as the node's *local name*, so a
`button { … }` type selector matches an authored `<button>` (spec §4.13). Both
halves come from the registry — [`name_binding::local_name_for`] owns the
policy, including the rule that an authored `type="…"` wins the same slot — and
the runtime bridge emits the identical property from the identical call.

A tag naming **no** built-in resolves by the node's kind: a `<div>` is the
`Container` box, and a `<Card>` is
[`user_component_id("Card")`](name_binding::user_component_id) — the id
`#[derive(Contract)]` computes for the same name, so a host that registers
`CardProps::contract().id` mounts the author's component under the authored
tag. Codegen never asks the §2.14 registry what `Card` is and could not: that
registry is a runtime value of `Box<dyn ComponentFactory>` trait objects,
while this runs during the host crate's own compilation. The name is what the
two sides share.

What that does not buy is a diagnostic: a tag naming a component nobody
registered emits an id nothing mounts, and — since no part of matching,
layout or paint reads a `ComponentId` — still renders as the same box it
silently rendered as before. See [`name_binding`]'s module docs for why that
rule cannot be stated once for both front ends.

# A binding is a Rust path

`{open}` in a `.okui` document emits the Rust expression `open`, verbatim.
There is no evaluator here and there does not need to be one: the emitted code
lands in a scope and **Rust resolves it**. A path that names nothing is a Rust
compile error, in the caller's build, pointing at the caller's code. A field
whose type cannot become a [`ViewValue`](obscur_view::view::ViewValue) — or,
for a body `{expr}`, cannot `Display` — is the same. That is the whole type
system of the expression language, borrowed rather than rebuilt.

This holds in `<div>{user.name}</div>` too, and that took a second commit: the
body case kept emitting the path's *spelling* as literal text long after the
attribute case stopped. See [`emit_text_content`].

This is also what makes §2.10's restrictions meaningful rather than
decorative: the allow-list polices what may appear in a path that is about to
become real Rust.

# Which scope, and the gap for a compiled file

For the inline `okui!` macro the scope is the **caller's**, and a binding reads
the caller's locals — that is the path the tests exercise end to end. For a
compiled file, [`document_to_view_fn`](ViewCodegen::document_to_view_fn) emits
a **nullary** `pub fn <name>() -> View`, so the only scope a binding can
resolve in is whatever the generated module has around it (a `const`, a
`static`, an import). A file whose bindings read per-call state does not
compile — the function has no parameters to read it from — and giving it them
needs a decision about the props signature that nothing here makes yet. The
emitted expression is right on both front ends; only the file front end's
function shape limits what a binding can name.

# Conditionals are real conditionals

`<If cond={open}>` emits `if open { … }`. `<ElseIf>` / `<Else>` following it
join the same chain, so exactly one branch renders — the point of having them.

Emitting a real `if` is why the children of a fragment are collected into a
`Vec` with `push` rather than written as a `vec![…]` literal: a conditional is
a *statement*, and there is no way to spell one as an element of a list
literal. The shape of the generated code follows from the language, not from
preference.

# Loops are real loops

`<For each={items} as={item} key={item.id}>` emits `for item in &items { … }`.
`as` names the loop variable, and it is a Rust binding like any other — which
is what lets the body's `{item.name}` be an ordinary path. The collection is
borrowed, so a `<For>` never consumes the caller's data.

`<Empty>` and `<Separator>` children are the §2.11 slots, not template nodes;
`<Loading>` is deliberately absent — see [`loop_slot`].

A `<VirtualFor>` is the exception: it emits **no** loop. It is a §4.9
performance contract, so it lowers to a single virtual-collection marker
carrying its row template, which the runtime realizes into a viewport-bounded
window — see [`emit_virtual_collection`].
