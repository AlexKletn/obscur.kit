Every face fontconfig hands back must be one the shaping stack can actually
parse. This is the `.ttc` guard's end-to-end consequence: a collection would
register with no `cmap` coverage and its text would silently vanish, so the
enumeration must never emit one.
