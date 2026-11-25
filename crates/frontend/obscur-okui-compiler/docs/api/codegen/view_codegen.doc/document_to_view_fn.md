Emits a `pub fn <fn_name>() -> View { … }` whose body constructs the
[`View`](obscur_view::view::View) equivalent to `document`, naming every
engine type through `root`.

The function is self-contained (fully-qualified paths, no `use` items);
drop it into a generated module the consumer `include!`s. `root` must be
the one that resolves in *that* crate — [`CodegenRoot::facade`] for an
application.
