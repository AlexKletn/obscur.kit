Returns the grid line edge if this value is a [`PropertyValue::GridLine`].

Returns a borrow so a [`GridLineValue::Named`] name is read without an
allocation; numeric / span / auto edges are the common case.
