The marker for a handler that takes **no** arguments: `FnMut()`.

It has no fields and no behaviour; it exists only so the two blanket impls of
[`IntoCommandHandler`] below are impls of *different* traits
(`IntoCommandHandler<WithoutEvent>` and `IntoCommandHandler<WithEvent>`) and so
cannot overlap. It lives beside the trait it tags rather than in its own file
because it is part of that trait's contract and meaningless apart from it.
