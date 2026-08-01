Mints a fresh runtime instance of the component registered under `id`.

Returns `None` if `id` is not registered, **or** if the component's
contract declares a [`CapabilityRequirement::Required`] capability that
has not been [`grant`](Self::grant)ed on this registry (spec §2.13,
§2.14) — an ungranted required capability refuses instantiation, so no
factory runs and no record is minted. The minted [`ComponentInstance`] is
in [`LifecycleState::Created`](crate::lifecycle::LifecycleState::Created).
