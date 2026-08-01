The replaceable port for presenting dialogs (spec §6.9 — dialog providers).

[`DialogProvider`] is the hexagonal port through which the domain asks the
platform to present a [`DialogRequest`](crate::dialog::DialogRequest) and
receives a [`DialogResult`](crate::dialog::DialogResult). A concrete provider
may be native, an external portal, or kit-rendered (spec §6.9), and providers
may be composed into a chain.

## Provider chains and loop detection (spec §6.9)

Because a kit-rendered system dialog can itself be *implemented over* a
`DialogProvider`, composing providers risks a provider re-entering itself and
recursing forever. The port surfaces [`DialogProvider::origin`] — a stable
identifier — precisely so a composing caller can record the origins already on
the chain and refuse to re-enter one. Loop detection is therefore the
*caller's* responsibility, using `origin()`; this pure port models the
identity but performs no runtime recursion of its own.
