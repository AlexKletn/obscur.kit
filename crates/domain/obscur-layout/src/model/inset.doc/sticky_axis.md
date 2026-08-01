Resolves one axis of a sticky node's used start position (top or left) from
its start/end insets, its static position, and the scrollport / containing
block edges along that axis. See [`Inset::sticky_offset`] for the semantics.

Returns the static position unchanged when neither inset is set on the axis,
so a non-sticky axis contributes a zero delta.
