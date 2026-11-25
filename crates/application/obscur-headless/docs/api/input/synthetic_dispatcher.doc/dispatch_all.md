Dispatches every `input` in order, returning each outcome.

Order matters and is preserved: a click is a `PointerDown` followed by a
`PointerUp` on the same node, and the runtime only completes a click when
it sees that pair.
