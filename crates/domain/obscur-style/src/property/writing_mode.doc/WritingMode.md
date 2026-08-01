The block-flow / line orientation a box lays its content out under
(`writing-mode`, css-writing-modes-4 §3).

This is the piece the block axis and the inline axis rotate around: it decides
which physical edge `block-start` names (the input
[`PropertyId::physical_under`](crate::property::PropertyId::physical_under)
needs beyond `direction`), and which physical axis normal flow stacks its
children along.

Only the three modelled values are represented; the sideways / `-rl` line
orientations are out of scope. The default is [`HorizontalTb`](Self::HorizontalTb),
the CSS initial, under which every mapping is byte-for-byte what it was before
`writing-mode` existed.
