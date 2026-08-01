The tag that spells a §2.12 slot outlet: `<slot name="header"/>`.

A `&str` constant rather than a bare literal because three places must agree on
it and cannot be allowed to drift: [`builtin_component_id`] binds the tag,
[`attr_symbol_on`] rebinds `name` on it, and the frontend's `check_outlet`
rule decides what is legal to write on it.

Lowercase only, and deliberately: an outlet is a *position marker*, not a
component, and both front ends classify a lowercase tag as an element (see
[`TagPosition`]). A `<Slot/>` in component position stays what any unrecognised
component tag is — [`user_component_id("Slot")`](user_component_id), mounting
nothing — because giving the outlet two spellings would mean the front ends'
element/component classification no longer decided anything about it.
