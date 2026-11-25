The largest single frame payload the transport will read, in bytes.

Sized to comfortably hold any well-formed DevTools message while still
rejecting a hostile or corrupt length prefix before allocating (spec §7.7).
