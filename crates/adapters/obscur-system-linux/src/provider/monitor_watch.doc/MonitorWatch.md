A watch built on a tool that streams a line per change.

Both watchable sources on this platform work this way — `gdbus monitor` for the
portal's `SettingChanged`, `gsettings monitor` for a GNOME schema — so the
process, the reader thread and the teardown are written once here rather than
twice.

It matches the rest of this adapter: a child process and `std`, no D-Bus stack
and no inotify crate.
