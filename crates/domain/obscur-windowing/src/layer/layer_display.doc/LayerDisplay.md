One display, as an application choosing where to put a surface sees it.

Two fields and a reason for each. The [`id`](Self::id) is what
[`LayerSettings::with_display`](super::LayerSettings::with_display) takes — the
platform's own handle, which stays meaningful across platforms because it is
whatever the backend that will place the surface already uses. The
[`name`](Self::name) is for a person: a connector like `eDP-1` or `DP-3`, to
show in a picker or accept on a command line. It is deliberately *not* the
identity, because a name that identifies a monitor on one platform identifies
nothing on the next.
