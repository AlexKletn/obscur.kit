A headless single-choice dropdown control (spec §2.6).

`Select` owns its list of `options`, the index of the currently
[`selected`](Self::selected_index) option (if any), whether the popup is
[`open`](Self::is_open), and a disabled flag. It bakes in **no** visual
theme — [`view`](Self::view) derives a semantic element carrying the open
and selected state as properties, a click event bound to `command`, and
each option as a [`Text`] child, rendered by the style layer keyed on
[`COMPONENT_ID`](Self::COMPONENT_ID).

The accessible role is [`SemanticRole::Menu`]: [`SemanticRole`] has no
dedicated list-box role, and a menu is the closest choice-list classification
in the common cross-platform set.

The state machine covers popup open/close ([`open`](Self::open),
[`close`](Self::close), [`toggle_open`](Self::toggle_open)) and committing a
choice ([`select`](Self::select), which validates the index and closes the
popup).
