Maps each [`ComponentId`] to its [`ComponentRegistration`] (spec §2.14).

Per §2.14, **static registration is the default**, and this type *is* that
static-default store: it is a plain owned value that holds the component
kinds a runtime knows about. Dynamic registration (from packages/plugins) is
an extension the plugin system layers on top by calling [`register`] at
runtime; the registry itself imposes no such policy.

# No global singleton

A registry is an ordinary owned value — construct one with [`new`] or
[`Default`]. Multiple runtimes may each own their own registry; there is no
process-wide instance (see the architecture's "avoid global singletons").

# Erased render is deferred

[`create_instance`] mints the plain runtime [`ComponentInstance`] record in
[`LifecycleState::Created`](crate::lifecycle::LifecycleState::Created) via the
registered factory. It does **not** drive an erased render — decoding
`ViewProperty` inputs into each component's typed `Props` and invoking
`render` is a later runtime concern (see [`ComponentFactory`](crate::component::ComponentFactory)).

[`register`]: ComponentRegistry::register
[`new`]: ComponentRegistry::new
[`create_instance`]: ComponentRegistry::create_instance

# Capability enforcement

The registry holds a set of **granted capabilities** (spec §1.8, §2.13,
§2.14). It starts **empty** — a fresh registry grants nothing. A component
whose contract declares a [`CapabilityRequirement::Required`] capability that
is not in this set does not instantiate: [`create_instance`] returns `None`.
This is the seam through which the host controls what untrusted components
may do (§9 forbids weakening host control over plugin capabilities). Grant
capabilities with [`grant`]; a component that requires none is unaffected and
instantiates exactly as before.

[`grant`]: ComponentRegistry::grant
