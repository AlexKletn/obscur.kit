Laying a view out far enough to assert against it, with only the facade.

A near-copy of the same helper in `checks/facade-consumer`. Duplicated rather
than shared on purpose: each fixture has exactly one dependency, and a second
path dependency — even on a sibling check — would be the crack the whole
arrangement exists to prevent.

A headless measurement harness an application can build from the facade.

`docs/external-consumer-audit.md` recommends lifting this into the repository
on its own merits: it answers a layout question with numbers instead of an
argument, and needs no display, window or GPU.
