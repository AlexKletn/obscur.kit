Where a synthetic input (or a headless query) is aimed.

Testing facilities target a node either directly, by a stable selector, or by
a viewport coordinate (spec §7.4). Stable selectors are independent of CSS and
do not affect layout or visuals (spec §7.5).

Resolution semantics ([`SelectorResolver`](crate::input::SelectorResolver)):
- [`SelectorTarget::Node`] names a node directly; it resolves only while that
  node is still live.
- [`SelectorTarget::TestId`] resolves by matching the node's `test-id`
  property. It is not a style attribute, so it affects no matching, layout or
  visual output (spec §7.5).
- [`SelectorTarget::Coordinate`] is *not* resolved here: it is aimed straight
  at [`UiRuntime::handle_pointer`](obscur_runtime::UiRuntime::handle_pointer),
  which hit-tests it exactly as it would a real pointer. Re-implementing that
  would give a second, divergent answer.
