A layout-dependent produced key (spec §6.7).

A logical key is what the active keyboard layout produces when a physical key
is pressed: the character `'a'`, `'/'`, or `'?'`, or a
[named](NamedKey) non-character key such as `Enter`. Logical keys are the
right surface for text entry and shortcuts, which follow the user's layout;
for layout-independent spatial bindings use
[`PhysicalKey`](crate::command::PhysicalKey).
