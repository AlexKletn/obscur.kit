Registers every entry from [`registrations`](Self::registrations) on
`runtime`.

Fails with the first [`RegistryError`] rather than continuing, because the
only way this fails is a host having already claimed a built-in id — and
silently keeping *both* answers to "what is a `<checkbox>`" would be the
worst outcome. A host replacing a built-in should skip this call (or use
[`KitBootstrap::without_builtin_components`]) and register its own set.

[`RegistryError`]: crate::view::component::RegistryError
[`KitBootstrap::without_builtin_components`]: super::KitBootstrap::without_builtin_components

# Errors

[`RegistryError::DuplicateComponent`] when a built-in's id is already
claimed on `runtime` — either by a host's own factory, or by a second call
to this function.

[`RegistryError::DuplicateComponent`]: crate::view::component::RegistryError::DuplicateComponent
