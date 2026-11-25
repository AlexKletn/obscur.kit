Installs `commands` as this runtime's **application** fallback — link 5,
the last of §2.6's chain (spec §2.6).

[`AppCommands`] is a shared handle: giving the *same* one (or a clone of
it, which shares the registry) to every window's runtime is what makes
the application level span windows, rather than being a second per-window
table. A runtime with no `AppCommands` installed has an empty fallback,
so the chain simply ends at the window level.
