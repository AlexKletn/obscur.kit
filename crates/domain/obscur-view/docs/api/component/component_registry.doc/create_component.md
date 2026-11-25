Mints a fresh renderable component value for `id`, or [`None`] when no
component is registered under it.

The [`create_instance`](Self::create_instance) counterpart: that returns
the bookkeeping record, this returns the value the runtime renders through
and which owns the component's local state.
