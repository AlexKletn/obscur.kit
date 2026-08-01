The output (monitor) a layer surface asks to live on.

# Why a name and not a handle

The domain has no Wayland objects in it and must not grow any, so a choice
of output has to be expressible as data. A connector name — `eDP-1`, `DP-2`,
`HDMI-A-1` — is what a user's configuration file says, what a compositor's
rules match on, and what survives being written to disk and read back, which
a protocol handle does not.
