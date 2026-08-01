Microseconds since the app started — the logical timestamp every pointer
event this runner dispatches carries.

The interaction domain never reads a clock (it takes the time as data), so
supplying one is the platform adapter's job, and nothing did it: every
event went out stamped `0`. That is not a cosmetic gap — the runtime's
multi-click test is *same target, within the click window, within the
radius*, and with every press at time `0` the window condition
(`0 - 0 <= 500ms`) is vacuously true. A press on the same spot was a
double click no matter how many seconds apart, so a file-manager row
clicked twice a minute apart opened the directory. Only the elapsed clock
makes the window mean anything.
