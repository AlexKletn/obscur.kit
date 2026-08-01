The rules that translate a click or range gesture into mutations of a
[`Selection`](super::Selection).

A policy pairs a [`SelectionMode`] with two flags:

- `allow_empty` — when `false`, at least one item must stay selected, so a
  toggle that would empty the set is suppressed.
- `toggle_on_click` — when `true`, a plain click toggles (Ctrl-like) rather
  than replacing the selection; a modifier can request the same behaviour
  per-click.

The policy is pure: [`apply_click`](Self::apply_click) and
[`apply_range`](Self::apply_range) mutate the passed-in selection and read no
external state.
