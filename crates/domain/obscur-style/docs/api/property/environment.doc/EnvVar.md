One CSS environment value, named by `env(<name>)` (spec §6.15; css-env-1).

These are **reported by the host, not chosen by the application**: the notch
a phone's screen is cut by and the on-screen keyboard's geometry are facts
about the device and the moment, and an app can only lay itself out around
them. That is why they arrive as a typed environment rather than as
properties an author sets.

A name this enum does not list is not an environment value here, and a
declaration using one is dropped as invalid rather than silently resolving to
zero — an unknown name is a typo or a value the kit does not model, and both
are better as a dropped declaration than as an inset that quietly is not
there.
