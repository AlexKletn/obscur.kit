The properties that select a **face**, and therefore decide what the `0` glyph
measures — the inputs
[`ChMetrics::zero_advance`](super::ChMetrics::zero_advance) needs (CSS Values 4 §6.1.2:
`ch` is the advance of `0` *"in the font used to render it"*).

`font-size` is not here: it selects a *size*, not a face, and the resolver has
to compute it separately anyway (it is the base its own node's `em` resolves
against). Both halves are set on the style before the port is asked.

Every entry must be a property that cannot carry a `<length>`, since these are
resolved before a [`LengthContext`](crate::property::LengthContext) exists. All three hold today — a family
list, a number, and a keyword respectively — and all three land in
`absolutize_value`'s identity arm.
