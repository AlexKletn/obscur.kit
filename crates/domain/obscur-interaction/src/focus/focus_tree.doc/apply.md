Resolves `request` against the current state, updating the focused node
and returning the [`FocusResult`].

- [`Node`](FocusRequest::Node): `NoTarget` if unregistered, `Rejected` if
  registered but not focusable, `Unchanged` if already focused, else
  `Focused`.
- [`Clear`](FocusRequest::Clear): `Cleared` if something was focused, else
  `NoTarget`.
- [`Next`](FocusRequest::Next) / [`Previous`](FocusRequest::Previous) /
  [`First`](FocusRequest::First) / [`Last`](FocusRequest::Last): pick from
  the computed [`TabOrder`] relative to the current focus (directional
  steps wrap), focusing the result or returning `NoTarget` when the tab
  sequence is empty.
