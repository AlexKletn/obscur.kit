Interned name of the option-label list property, and the one prop of this
control the Rust builder had no way to state before it needed to.

The value is the labels joined by [`OPTION_SEPARATOR`]. A `ViewValue` has no
list variant, so a list of options cannot be one property without an encoding,
and every encoding has a delimiter some label could contain — which is why
this is a *second* spelling beside the authored `options` attribute rather
than the same one: markup gets a readable comma-separated list it can type,
and the builder gets a separator no typed label contains, so a
`Select::with_options(["a,b"])` still renders one row.

Emitted only when the control has options, so absence means "no options" and
a present empty string means "one option whose label is empty". Joining an
empty list and joining `[""]` both give `""`, and those two are different
trees.

[`OPTION_SEPARATOR`]: Self::OPTION_SEPARATOR
