`box-shadow` lowering: the offset/blur/spread/colour `<shadow>` list (§5.2).

`filter: drop-shadow()` reads its lengths through the same
[`shadow_length`] and produces the same [`BoxShadow`], which is why that
reader is `pub(super)` rather than private — the two properties are different
grammars over one modelled value.
