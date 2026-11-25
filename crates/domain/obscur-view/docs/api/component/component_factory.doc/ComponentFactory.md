Object-safe description of how to instantiate one kind of component.

The author-facing [`Component`](crate::component::Component) trait is
intentionally **not** object-safe (it has an associated `Props` type and a
`Self`-typed API), so a heterogeneous [`ComponentRegistry`] cannot store
`Component` values directly. `ComponentFactory` is the erasure seam: each
concrete component kind is fronted by a factory that reports its identity,
hands out an owned copy of its [`ComponentContract`], and mints fresh runtime
[`ComponentInstance`] records — all behind a `dyn` handle. Because every
method here is object-safe, factories are stored as
`Box<dyn ComponentFactory>`.

The [`core::fmt::Debug`] supertrait is required so that a boxed factory (and,
transitively, the [`ComponentRegistration`](crate::component::ComponentRegistration)
that owns one) is `Debug` without a hand-written impl.

# The two halves of an instance

Instantiating a component mints **two** values, deliberately kept apart:

* [`create_instance`](Self::create_instance) mints the
  [`ComponentInstance`] — plain bookkeeping (identity, lifecycle state,
  subscription scope) that stays `Debug` and inspectable;
* [`create_component`](Self::create_component) mints the author's component
  value behind an [`ErasedComponent`] handle — that is where the component's
  [`LocalState`](crate::state::LocalState) fields live, so it is the value a
  runtime must keep alive across frames to preserve state.

Splitting them keeps the record free of the erased object (and therefore
`Debug`), while still giving a runtime everything it needs to render.
