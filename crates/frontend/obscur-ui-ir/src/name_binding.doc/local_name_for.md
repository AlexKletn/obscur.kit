The **local name** (CSS *type*) the tag contributes to the node it lowers to,
or [`None`] when it contributes none.

This is the whole of "a base element, as in HTML": a `<button>` an author
wrote must be what a `button { … }` type selector matches (spec §4.13, "type,
class, id, attribute, descendant/child/sibling combinators"), and the matcher
reads an element's local name from the reserved [`TYPE`] property — nothing
else. So both lowering paths append a `TYPE` property nobody authored, and
this function is the single definition of when and to what, because the two
paths must emit the same bytes.

# `type` is two channels, and the author owns it

[`TYPE`] is overloaded, and honestly so: it is *both* the element's local name
(this function's implicit value) *and* the reserved symbol an authored
`type="…"` attribute binds to — and HTML really does write
`<button type="submit">`, so the collision is not hypothetical. There is one
slot and two writers, so the rule is that the **author wins**: given any
authored attribute that binds to `TYPE`, this returns [`None`] and the node
carries the authored value alone. `<button type="submit">` is therefore an
element whose CSS type is `submit`, which is the price of the overload and not
a thing to paper over: an author who wants both spellings out of one node has
to say so with a class. (`authored_attributes` is asked for by *name*, not by
resolved symbol, so the caller cannot forget to route it through
[`attr_symbol_on`] — this function does that itself, on the same tag.)

# A component tag gets none

`<Card/>` is not an element; it names a component, whose own render produces
the elements — and each of *those* carries its own local name (that is exactly
what `obscur_widgets::Button::view` sets `TYPE` for). Naming the *host* node
`Card` would invent a CSS type no vocabulary has, and a costly one: a type
selector is matched ASCII-case-**insensitively**
(`SelectorMatcher::matches_simple`), so a `<Card/>` and a hypothetical `<card>`
element would be indistinguishable to the cascade. A component's node
therefore stays anonymous to type selectors and is styled by class, by id, or
by whatever its render emits — the same rule §2.12 slot projection already
implies, where the host node is a seam rather than a box the author styles.

# And so does the outlet

[`OUTLET_TAG`] is written in element position but is not an element either: it
is a *position marker*, substituted away by `project_slots` before the tree is
reconciled (and reported as stray if it is not), so no retained node ever
carries `slot` as its type and no type selector could ever match one. Naming it
would be dead weight — and worse than dead: an authored `<slot name="header"/>`
must be **the same node** a component's own `ViewNode::slot("header")` builds,
"one outlet, two spellings, not two outlets"
(`okui_slot_projection.rs::an_authored_slot_tag_is_the_outlet_on_the_macro_front_end`
asserts that equality outright). An extra property on one of the two spellings
would split them.
