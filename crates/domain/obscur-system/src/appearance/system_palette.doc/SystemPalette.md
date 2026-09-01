The colours a desktop is willing to name.

Every field is optional and that is the substance of this type, not a
convenience. Reading a GNOME session through `gsettings` yields **no colours at
all** — the settings name a theme, and the theme's palette lives inside GTK,
reachable only by linking it. A KDE session's `kdeglobals` states the whole
thing as plain INI. One unified shape, two very different amounts of truth in
it, and a caller can tell which it got.

Three pairs, because that is the distinction every desktop toolkit draws:
the *window* chrome, the *view* a document sits in (often lighter, or white
under a grey window), and the *selection*.
