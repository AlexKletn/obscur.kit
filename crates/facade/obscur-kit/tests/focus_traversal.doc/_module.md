Keyboard focus traversal, reached the way an application reaches it.

`TabOrder`, `FocusTree` and `SpatialNavigation` had lived in the domain tier
with **zero callers anywhere in the workspace** — not one production site, not
even a test constructing a `TabOrder`. `docs/api/06-input.md` had said so for a
long time under a heading that read "Tab does not move focus", which meant a
keyboard user could reach exactly one control: whatever a pointer had last
pressed.

Three things had to be true at once for Tab to be worth having, and the tests
below are one per thing: the ring must visit controls and not every box on the
screen; the move must be visible, or the user cannot see where they have been
taken; and the application must be able to refuse it, or a grid can never take
Tab for itself.

Not a `spec_*` file: the spec-derived suites are an oracle written without
reading this repository.
