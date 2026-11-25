A `grid-area: <custom-ident>` named-area reference on a grid item: the area
name to resolve against the item's container `grid-template-areas`. The name
is kept **as a string** (not an interned symbol) because it is matched
against the container template's area names in the runtime, which does not
share this crate's interner (spec §4.2). The numeric `grid-area:
a / b / c / d` line form does not produce this value; it expands to the four
[`GridLine`](PropertyValue::GridLine) longhands instead.
