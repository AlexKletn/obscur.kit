Mirror of `obscur_view::view::SLOT_OUTLET` — the [`ComponentId`] of a §2.12
**slot outlet**, the position a component's render names for its authored
children to be projected into.

This is what [`OUTLET_TAG`] binds to, and it is the reason the outlet needed no
new syntax on either front end: `<slot name="header"/>` is an ordinary element
tag with an ordinary attribute, and the two [`name_binding`](self) rules
(`slot` → this id, and `name` → [`SLOT`] *on this tag*) are the whole feature.

It sits in a reserved range of its own (`0x00B0_xxxx`), below the primitive
(`0x00C0_xxxx`) and widget (`0x00D0_xxxx`) ranges and clear of the top-bit
namespace [`user_component_id`] hashes into.
