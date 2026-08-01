A handle to the installed bridge, or [`None`] when none is.

Hands back a clone rather than a borrow so the caller's push can hold
`&mut` on this publisher while the bridge is borrowed: the bridge is
shared with the host by construction, so this is a refcount bump rather
than a copy of anything.
