Renders the header row and, while open, the option list — [`Select::contents`],
the same body [`Select::view`] installs under its own root, so a mounted select
and a built one are the same tree.

The context is unused: nothing here reads a signal or asks for a re-render. A
select re-renders when its host node's props change, which the component pass
already tracks.

[`Select::contents`]: super::Select
[`Select::view`]: super::Select::view
