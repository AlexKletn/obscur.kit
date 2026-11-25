A Linux [`NotificationProvider`] backed by `notify-send` (spec §6.12).

The provider holds the tool command name (defaulting to `"notify-send"`) and
the last badge value set through [`Self::set_badge`]. [`Self::permission`]
scans `PATH` for the tool without executing it — so it never pops a
notification — and [`Self::post`] spawns the tool fire-and-forget.
