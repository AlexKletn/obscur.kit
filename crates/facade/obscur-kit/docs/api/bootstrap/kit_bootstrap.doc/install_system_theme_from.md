Read the desktop's appearance and install it, saying so when there is none.

The call a standalone application wants: it asks the provider, reports what it
found, and installs the variables — one line instead of three.

**A desktop with nothing to read is not an error.** No portal, no `gsettings`,
no `kdeglobals` is a legitimate machine — a bare X session, a container, a
minimal Wayland compositor — and there is nothing to fail at. Nothing here
returns a `Result` and nothing panics: the appearance comes back empty, no
`--obscur-sys-*` variable is defined, and the application's own stylesheet is
the whole answer, which is exactly what it would have been without this call.

What does happen is **one warning**, because the alternative is worse. Silence
would make "my colours did not follow the system" indistinguishable from a
defect in the application, and this codebase has spent enough time on defects
that turned out to be a subsystem reporting into nowhere.

Three things it may say, and only the first two are warnings:

| code | when |
|---|---|
| `platform.appearance.no_source` ⚠ | nothing on this machine can be read |
| `platform.appearance.empty` ⚠ | a source answered, and stated nothing |
| `platform.appearance.no_palette` ℹ | a source answered without colours — the ordinary GNOME case |

The note is `Info`, below the default stderr floor, because it is the expected
outcome on a common desktop rather than something wrong.
