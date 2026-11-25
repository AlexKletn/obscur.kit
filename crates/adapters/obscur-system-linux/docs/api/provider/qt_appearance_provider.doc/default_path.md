`$XDG_CONFIG_HOME/kdeglobals`, or `~/.config/kdeglobals`.

Falls back to a relative path when neither variable is set, which will not exist
and will answer `Unavailable` — the correct outcome for an environment with no
home directory.
