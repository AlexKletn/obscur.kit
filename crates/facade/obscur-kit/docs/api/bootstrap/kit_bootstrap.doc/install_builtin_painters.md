Registers the shipped widgets' canvas painters on `runtime`, unless this
bootstrap was built [`without_builtin_components`].

[`install_builtin_components`] already calls this — a control and the routine
that draws it are one delivery, and the same opt-out governs both. It is public
for the host that opted out of the component factories (because it mounts its
own) but still authors `Progress`/`Slider` views through the Rust builders,
which bind the built-in painters by symbol regardless of what is registered as
a component.

Registering a painter cannot fail: a key is overwritten rather than claimed, so
this never reports the duplicate that [`install_builtin_components`] can.

[`without_builtin_components`]: KitBootstrap::without_builtin_components
[`install_builtin_components`]: KitBootstrap::install_builtin_components
