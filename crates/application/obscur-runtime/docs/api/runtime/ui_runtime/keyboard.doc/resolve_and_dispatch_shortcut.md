Resolves the chord against §2.6's shortcut bindings and dispatches what it finds.

Runs whether or not anything is focused. That used to be impossible: the whole
body of `handle_key` returned early on `focused == None`, so a chord aimed at the
window surface or the application — the last two links of §2.6's own route — was
unreachable, and `Ctrl+S` did nothing until the user had clicked something first.
