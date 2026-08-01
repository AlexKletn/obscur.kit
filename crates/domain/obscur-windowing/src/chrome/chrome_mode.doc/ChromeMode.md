How the window frame and its controls are provided (spec §6.4).

This selects who draws the window's decorations. Dynamic switching between
modes is allowed where the platform supports it; where it does not,
[`ChromeMode::SystemFrame`] is the safe fallback and is therefore the
default.
