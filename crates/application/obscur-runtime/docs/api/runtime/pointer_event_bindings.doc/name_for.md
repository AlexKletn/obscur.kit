The reserved event name a pointer event of this kind is dispatched under, or
`None` for the crossing kinds.

A crossing kind has no name of its own because `pointerenter` / `pointerleave`
describe a node the hover chain crossed rather than a kind of report, and the
hover transition — which sees these two kinds' crossings along with every other
— is what emits them.

`wheel` is not answerable here at all: there is no `PointerEventKind::Wheel`,
because a notch is not a pointer report. It shares the gate and nothing else,
and `handle_scroll` asks for its bit by name.
