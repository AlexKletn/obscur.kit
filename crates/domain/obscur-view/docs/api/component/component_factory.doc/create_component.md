Mints a fresh component value for this kind, behind the object-safe
[`ErasedComponent`] handle.

This is the renderable half: the returned value holds the component's
[`LocalState`](crate::state::LocalState) fields, so a runtime keeps **one**
of these alive per mounted instance and re-renders through it. Dropping it
drops the component's local state and scoped subscriptions — which is
exactly the spec's "automatic cleanup" (§2.1).

A typical implementation is `Box::new(MyComponent::default())`; the
blanket impl on [`ErasedComponent`] makes any `Component` whose
`Props: FromProps` usable here without extra glue.

[`FromProps`]: crate::component::FromProps
