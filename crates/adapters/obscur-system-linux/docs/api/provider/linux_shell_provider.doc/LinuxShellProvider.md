A Linux [`ShellProvider`] backed by `xdg-open` (spec §6.11).

The provider holds the opener command name (defaulting to `"xdg-open"`) so a
caller may substitute an alternative freedesktop opener. Capability probing
([`Self::can_open`]) scans `PATH` for the opener without executing it, so it
never pops a GUI; [`Self::open`] spawns the opener and does not wait.
