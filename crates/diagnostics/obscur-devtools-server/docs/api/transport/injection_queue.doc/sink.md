Returns the push end to hand to a
[`DevToolsEndpoint`](crate::endpoint::DevToolsEndpoint).

The sink holds a [`Weak`] reference, so it keeps nothing alive: it is
usable exactly as long as *this* queue — the drain end — is.
