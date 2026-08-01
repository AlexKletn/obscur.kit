End-to-end: an authored `<h1 slot="header"/>` **reaches the component's
header outlet**, and reaches no other one (spec §2.12, §2.13).

# What this closes

§2.12's slots were declared and enforced but routed nowhere. A component
contract could name a slot, both front ends checked an author's `slot="…"`
against that declaration — and then `<Card><h1 slot="header"/></Card>`
rendered Card and **dropped the h1**, because a component's output named no
position for it to go to. `okui_slots.rs` said so out loud: "What does not
happen is *projection*."

The missing half was the outlet, not the signature.
[`ViewNode::slot("header")`](obscur_kit::view::view::ViewNode::slot) is a
component's way of saying *where* its `header` content goes; the reconciler
keeps *what* goes there (`RetainedNode::slot_content`); and
`UiRuntime::render_components` joins them with `project_slots` before the
component's output is committed.

# Why `Component::render` did not change

Because a slot is a **declaration**, and handing a component its children
would leave the declaration inert. §2.12 says a component *declares* named
slots and §2.13 exports `slots` as a field of its contract — separate from
`props`. A `render(ctx, props, children)` signature answers a different
question: it gives a component the *content* but not the *position*, so every
component would re-implement grouping-by-slot-name in Rust and the contract's
declared slots would still route nothing. That is this repo's signature defect
— a declaration nobody acts on — bought at the price of an authoring-trait
change.

Children-in-`props` loses on §2.13 (which lists props and slots separately)
and on the type: a prop is a `ViewValue`, a scalar/text union that the CSS
matcher and `StyleAttributes::extract` walk, and a subtree does not belong in
it. Children-in-`ctx` loses on §3.1, which enumerates how state enters a
component (typed props, read-only sources, local state) and does not include
the scope handle — and it would let a component that declares no slot read
content anyway, making the contract's `slots` field advisory.

# What is NOT claimed

* **No shipped component declares a slot**, because no shipped *widget is a
  `Component`* at all — every `impl Component` in this repo is a test's. Slots
  are toolkit API for a host's own components, exercised here exactly as a host
  would. `Dialog` still takes one anonymous child list and is a view-builder,
  not a component; giving it slots means making it a component first.
* **The outlet's name is not checked against the component's declaration.**
  A component that renders `<slot name="hedaer"/>` silently receives nothing.
  The declaration is on the contract, which `ComponentStore` holds at runtime;
  no front end can reach it (see below) and nothing at runtime checks it.
* **The outlet is not checked for being inside a component at all,
  at compile time.** It cannot be: `render` returns a `View` and `okui!`
  produces a `View`, so the same expansion is correct in one function and stray
  in the next, and no proc-macro can tell those apart. The stray case is caught
  at runtime instead, where the answer is knowable — see the fail-closed tests
  below.

# The authored spelling, and the obstacle that was not one

`<slot name="header"/>` used to be an unknown element tag on both front ends —
a generic box with a hashed `name` prop — and the audit recorded why it could
not cheaply become one: "`name_binding::attr_symbol` binds an attribute by its
own spelling with no idea which tag it sits on". True of the function; false as
a conclusion. Of its three production callers, the two that lower a *document*
hold `node.tag()` in the scope they call it from, and both were already
threading that node's `TagPosition` out of it. The tag was one argument away,
not absent. (The third — `#[derive(Contract)]`, hashing a struct field — really
has no tag, which is why the tagless `attr_symbol` survives alongside the new
`attr_symbol_on`.)

So the spelling is two `name_binding` rules and no new syntax in either
parser: `slot` binds to `SLOT_OUTLET`, and `name` binds to `SLOT` *on that tag
only*. Both front ends gain it together because both reach `ViewNode` through
those two callers — which is the `<VirtualFor>` house rule holding by
construction rather than by discipline.
