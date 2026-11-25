Mirrors `obscur_view::view::RESERVED_EVENTS` — every `on:` name the runtime
actually dispatches.

Any other name hashes through [`fallback_symbol`] and binds a command nothing
will ever fire, so this is what lets a front end tell an author that
`on:keydwon="…"` is a typo rather than accepting it in silence.
