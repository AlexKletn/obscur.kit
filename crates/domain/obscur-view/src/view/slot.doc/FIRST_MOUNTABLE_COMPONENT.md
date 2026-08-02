The lowest [`ComponentId`] a component can be registered under: the start of
the widget range (`0x00D0_0000`).

Everything below it is a position rather than a component — the reserved
runtime constructs (`0x00B0_xxxx`: the outlet, the virtual collection) and the
primitives (`0x00C0_xxxx`), none of which the registry ever mounts anything
on. Everything at or above it is a shipped widget or, with the top bit set, a
user component. [`may_project_content`] is its only reader.

[`ComponentId`]: obscur_identity::ComponentId
