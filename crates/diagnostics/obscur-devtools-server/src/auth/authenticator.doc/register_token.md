Registers `token` as granting `scopes`.

Re-registering the same token replaces its scopes. The stored scopes are
the *requested* grant; any production exclusions are applied at
[`authenticate`](Self::authenticate) time.
