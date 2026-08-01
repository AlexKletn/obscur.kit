The test harness the runtime's own suite is written on (spec §2.4).

Five things live here and nothing else: **semantic selectors**, **fixtures**,
**mock providers**, **snapshot normalization** and **high-level assertions**, plus
the plugin test host facade. **Production logic does not live here** — every
entity below exists to let a test say what it means in one line instead of ten,
and the moment one of them starts *deciding* something rather than *arranging* or
*reading* it, it belongs in the crate it came from.

The dependency runs one way and it is worth stating, because it looks backwards
at first glance: this crate takes an ordinary dependency on `obscur-runtime`,
because a fixture has to build a real runtime, and `obscur-runtime` takes a
**dev**-dependency on this crate, because its own unit tests use the same
fixtures its integration tests do. That is not a cycle — a dev-dependency is a
separate build of a separate target — and it is what stops the harness from being
duplicated on either side of the crate boundary.

Everything a fixture reaches for is the runtime's **public** API. That is a
constraint, not an accident: an integration test in `tests/` can reach nothing
else, so a helper that needs more is telling you either that the thing it needs
deserves to be public on its own merits, or that its tests belong beside the code
in `src/`.
