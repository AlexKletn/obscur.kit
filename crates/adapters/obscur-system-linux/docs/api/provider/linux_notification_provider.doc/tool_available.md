Returns whether an executable named `cmd` exists on `PATH` (spec §2.7).

The probe scans the `PATH` environment variable, splitting on `:`, and tests
each `dir/cmd` with [`std::fs::metadata`]. It deliberately does **not** execute
the tool, so capability queries are deterministic and never pop a notification.
