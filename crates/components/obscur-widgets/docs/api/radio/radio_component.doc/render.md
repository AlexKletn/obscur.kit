Renders the indicator sub-part and the label — [`Radio::contents`], the same
body [`Radio::view`] installs under its own root, so a mounted radio and a
built one are the same tree.

The context is unused: nothing here reads a signal or asks for a re-render. A
radio re-renders when its host node's props change, which the component pass
already tracks.

[`Radio::contents`]: super::Radio
[`Radio::view`]: super::Radio::view
