Renders the indicator sub-part and the label — [`Checkbox::contents`], the
same body [`Checkbox::view`] installs under its own root, so a mounted
checkbox and a built one are the same tree.

The context is unused: nothing here reads a signal or asks for a re-render.
A checkbox re-renders when its host node's props change, which the
component pass already tracks.

[`Checkbox::contents`]: super::Checkbox
[`Checkbox::view`]: super::Checkbox::view
