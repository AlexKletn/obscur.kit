Appends a [`PaintCommand::PushTransform`] and increments the nesting depth.
Everything drawn until the matching [`pop_transform`](Self::pop_transform)
is mapped by `transform`. Returns `&mut Self` for chaining.
