The region a layer surface asks other surfaces to keep clear of.

This is what makes a panel *work*: a maximised window stops at the panel's
edge instead of underneath it. It is modelled as three cases rather than as a
signed number because the protocol's `-1` is not "minus one pixel" — it means
something categorically different, and a bare `i32` invites writing `-1` by
accident and getting a surface that ignores every other panel on the screen.
