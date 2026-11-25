Opts this bootstrap out of registering the shipped controls, so
[`install_builtin_components`](KitBootstrap::install_builtin_components)
becomes a no-op.

The reason to reach for it is that a host wants its OWN `<checkbox>`: a
built-in id can be claimed once, so a host that registers a replacement must
first stop the default from claiming it. Opting out leaves every authored
`<checkbox>` an element with the right id and no contents — which is what
the whole toolkit did before the built-ins existed.
