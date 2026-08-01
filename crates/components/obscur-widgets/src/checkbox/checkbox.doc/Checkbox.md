A headless tri-state checkbox control (spec §2.6).

`Checkbox` owns its `checked` state as an [`Option<bool>`]: `Some(true)` is
checked, `Some(false)` is unchecked, and `None` is the *mixed* /
indeterminate state a parent can hold while its descendants disagree. It
bakes in **no** visual theme — [`view`](Self::view) derives a
[`SemanticRole::Checkbox`] element whose checked state is encoded as a
tri-state integer property (see [`view`](Self::view)) plus a change event
bound to `command` and a single [`Text`] label child, rendered by the style
layer keyed on [`COMPONENT_ID`](Self::COMPONENT_ID).

The state machine is [`toggle`](Self::toggle): disabled is a no-op, `None`
(mixed) advances to `Some(true)`, and a definite value flips.
