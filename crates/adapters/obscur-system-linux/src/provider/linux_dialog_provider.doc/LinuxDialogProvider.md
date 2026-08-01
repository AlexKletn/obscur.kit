A Linux [`DialogProvider`] backed by `zenity` (spec §6.9).

The provider holds the tool command name (defaulting to `"zenity"`).
[`Self::supports`] scans `PATH` for the tool without executing it — so it never
pops a GUI — and [`Self::present`] runs the tool with a request-specific argv.
