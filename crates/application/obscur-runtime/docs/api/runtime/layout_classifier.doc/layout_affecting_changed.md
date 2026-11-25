Whether any **layout-affecting** computed property differs between the
retained style `old` and the freshly resolved `new` (spec §4.3).

A missing `old` (a node with no retained style) counts as changed. Both
directions are compared so a property that was *dropped* this frame — not just
one that changed value — still dirties layout. Paint-only properties
([`PropertyId::is_paint_only`](obscur_style::property::PropertyId::is_paint_only))
are ignored: they provably cannot move a box.
