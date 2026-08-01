Appends a [`PaintCommand::PopLayer`] and decrements the nesting depth
(saturating at zero). The command is always recorded, even if the stack
is already empty. Returns `&mut Self` for chaining.
