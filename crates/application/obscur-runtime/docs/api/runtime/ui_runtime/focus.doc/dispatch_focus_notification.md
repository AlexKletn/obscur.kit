Dispatches one half of the pair to the nearest binding at or above the node that
gained or lost focus.

The binding is resolved through the **ancestor chain**, the same way
`dispatch_raw_key` resolves `keydown` on the same focused node — and not the way
`dispatch_crossing` resolves a hover crossing on the crossed node alone. That is
a deliberate deviation from the DOM, where `focus` and `blur` do not bubble, and
[`EVENT_FOCUS`](obscur_view::view::EVENT_FOCUS) states why: the DOM focuses
elements, this runtime focuses whatever the press hit, and a press on a field's
glyphs hits the field's **text leaf**. A rule that fired only for the exact
focused node would make `<input on:focus="…">` run or not depending on whether
the user clicked the text or the padding beside it.

Walking carries no duplicate-delivery risk, which is what separates this from a
crossing: exactly one node gains focus and one loses it, so the walk delivers
exactly once per half. A crossing set is a whole chain, which is why that one
must not walk.

The outcome is dropped. Focus has already moved, so `prevent_default` has
nothing to suppress here.

# Cost

One ancestor walk per half, so at most two per focus change — and a focus change
happens at most once per pointer press. That is human rate, on the path that
already hit-tests, recomputes pseudo-states and schedules a frame, so it needs
no [`PointerEventBindings`](crate::runtime::PointerEventBindings)-style
tree-wide gate the way `pointermove` does. Reasoned rather than measured, and
the reasoning is the rate: nothing here is on the report-rate path.
