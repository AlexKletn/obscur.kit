End-to-end: an authored `<Card/>` **is** the author's component (spec §2.13,
§2.14) — on both front ends.

# What used to happen

`name_binding::component_id_for` resolved every tag it did not recognise to
the generic `Container` primitive. So `<Card/>` was not a mis-checked
component and not a rejected one: it was a **box**. No diagnostic, no error,
no mount — an author wrote a component and got an empty div. §2.13's contract
validation "skips any tag with no registered contract" was checking nothing
because the tag was never going to be a component in the first place.

# What closes it

The id a component tag binds to is derived from its **name**
([`name_binding::user_component_id`]), which is the very scheme
`#[derive(Contract)]` already computes `ComponentContract::id` with. The
registry (`ComponentRegistry`) is a *runtime* value holding
`Box<dyn ComponentFactory>` trait objects and cannot be consulted by a code
generator that runs while the host crate is still compiling — but it does not
need to be, because both sides can compute the same id from the same string.
`derived_contract_id_agrees_with_the_authored_tag` below is the drift guard on
exactly that agreement.

The runtime half was already real and already tested (`ComponentStore` mounts
an instance for any node whose `ComponentId` is registered and splices its
`render` output beneath it); the front ends were emitting the wrong id into
it.

# Why every end-to-end test here carries a sibling box

Because the first draft of them did not, and passed with the fix reverted.
The derive resolves a name through the *same* `component_id_for` the front
ends do — so in a build where `Card` falls back to `Container`, the host
registers Card under the container id, `<Card/>` emits the container id, they
agree, and the component mounts and renders "card:Hi" exactly as it should.
What that build also does is mount a Card for every `<div>` on the page.

Agreement between the two sides is therefore not the property under test —
*identity* is. The `<div>`/`<span>` siblings are what tell the two apart:
exactly one of the three nodes may mount.

# What is still open, and is not claimed here

* **An unregistered component tag is still silent.** `<Crad/>` (a typo) now
  binds to `user_component_id("Crad")`, nothing is registered under it, and it
  mounts nothing — no diagnostic. It is not even visibly different: nothing in
  matching, cascade, layout or paint reads a `ComponentId`, so it still renders
  as the box it silently rendered as before. The file front end *could*
  diagnose this (its `Compiler` takes a `ContractRegistry`); the `okui!` macro
  provably cannot (`CardProps::contract_surface()` is a function that runs when
  the built crate runs, and the macro runs while that crate is still being
  compiled), so the rule cannot be shared and is not added to one front end
  alone. See `an_unregistered_component_tag_mounts_nothing`, which pins the
  residue rather than hiding it.
* **A component that declares no outlet still drops its authored children**,
  and this file's `Card` is one: it declares no slot and renders no
  `ViewNode::slot(…)`, so the `<span slot="header"/>` beneath it reaches no
  position and does not appear. That is §2.12 working, not §2.12 missing — a
  slot is a declaration, and Card declares none.
  `authored_children_under_a_component_that_declares_no_slot_are_dropped` pins
  it; `okui_slot_projection.rs` is where a component that *does* declare one
  receives its content.

# Why there are computed-style assertions here now

Because every test in this file asserted on rendered **text**, and text is
exactly the half of "a component works" that cannot fail when its output is
invisible to the cascade. The external consumer audit's finding 2-1 —
"a component-rendered subtree is matched by no selector" — went straight
through this file: `Card` mounts, `Card` renders, `card:Hi` appears, and the
author's `.card { … }` reaches nothing. `component_output_is_styled.rs` is
the dedicated pin on §4.15's two halves; the two tests at the end of this
file are the habit itself, on the front ends, so a future front-end change
that quietly stops a component's output from reaching the style pass fails
*here* rather than in an external application.
