The channel a fault the bridge discovers is reported on, plus the identity it
is reported under.

Not a co-equal entity with [`ResourceBridge`] and so not its own file (spec
§3.1): it has no meaning apart from the bridge, and exists only so the four
operations that can *fail* name what they need instead of reaching into a
shared `self` for it. The runtime builds one at each such call site.
