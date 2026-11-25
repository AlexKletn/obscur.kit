The state flags of an accessibility node (spec §7.1 state, selection, focus).

These flags carry the changeable condition of a node — whether it can take
focus and currently holds it, whether it is disabled, programmatically
hidden, selected, busy, required, invalid, or read-only — plus two tri-state
conditions expressed as [`Option<bool>`]: [`checked`](Self::checked) (where
`None` means the node is not checkable) and [`expanded`](Self::expanded)
(where `None` means it is not expandable).

The [`hidden`](Self::hidden) flag means *programmatically* hidden (e.g. an
`aria-hidden` node), distinct from the decorative hiding tracked on the
owning [`SemanticNode`](super::SemanticNode).

The many independent boolean fields mirror the flat set of accessibility
state flags in spec §7.1 and are deliberately kept as separate fields rather
than folded into a bitflag type.
