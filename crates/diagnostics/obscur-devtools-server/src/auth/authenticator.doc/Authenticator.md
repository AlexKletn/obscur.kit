A token → [`ScopeSet`] registry that authenticates DevTools sessions.

The endpoint is **disabled by default** (spec §7.7): a freshly
[`new`](Self::new)'d authenticator has an empty registry, so
[`authenticate`](Self::authenticate) rejects every token until the host
explicitly [`register_token`](Self::register_token)s one. Tokens are opaque
local capabilities and are never logged.

A [`production`](Self::production) authenticator additionally **strips the
`Control` (and `Inject`) scopes** from every granted set, so a token that
would grant full access in development can only ever observe in production —
the dangerous capabilities are removed regardless of what was registered.
