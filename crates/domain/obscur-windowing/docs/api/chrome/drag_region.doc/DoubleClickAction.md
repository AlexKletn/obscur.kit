The action taken when the user double-clicks a drag region (spec §6.4).

This is the platform-conventional title-bar double-click action.
`DoubleClickAction` is a tightly-coupled part of [`DragRegion`](super::DragRegion): it only has
meaning as the action of a drag area, so it is colocated here rather than
given its own module.

The enum's own default is [`DoubleClickAction::None`] so that a bare value
is inert. A freshly constructed [`DragRegion`](super::DragRegion) likewise starts with `None`;
callers pick the platform-conventional [`DoubleClickAction::Maximize`]
explicitly via [`DragRegion::with_double_click_action`](super::DragRegion::with_double_click_action).
