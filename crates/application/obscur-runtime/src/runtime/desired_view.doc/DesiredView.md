The desired tree a frame reconciles, resolved lazily so a reactive frame can
build it **after** phase 3 (§3.4 "dispatch input/events") — an input handler
that commits state must have run before the view is rebuilt from that state,
or its mutation lands a frame late.
