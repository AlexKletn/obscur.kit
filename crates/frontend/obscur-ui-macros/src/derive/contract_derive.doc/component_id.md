The [`ComponentId`] raw value a component **name** binds to, resolved through
the shared `name_binding` registry — the component-side mirror of
[`name_symbol`].

This is the id an authored `<Name/>` lowers to on both front ends, which is
the only value a contract's `id` can usefully be: a host registers the
component under `MyProps::contract().id`, and the runtime mounts it when a
node carries that same handle. A separate hash here would agree with the tag
only by luck — and for a name that happens to be a built-in's (`Button`), not
even that.

[`ComponentId`]: obscur_identity::ComponentId
