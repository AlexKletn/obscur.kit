One entry in the component registry (spec §2.14).

It aggregates exactly what §2.14 lists a registry stores, without
duplication: the stable [`ComponentId`], the [`ComponentContract`] — which
already carries the contract schema, the component's capability requirements,
and its diagnostics metadata (spec §2.13) — the [`ComponentVersion`] of that
schema, and the boxed [`ComponentFactory`] used to instantiate the kind.

# `Debug` and the boxed factory

This type owns a trait object (`Box<dyn ComponentFactory>`), so it cannot
derive `Clone`, `PartialEq`, or `Eq`. It *does* derive [`Debug`]: that is why
[`ComponentFactory`] carries a [`core::fmt::Debug`] supertrait — the boxed
factory is `Debug`, so no hand-written impl is needed.
