Creates a production authenticator that strips the `Control` and
`Inject` scopes from every granted set (spec §7.7).

Registered tokens can still grant `Read`/`Subscribe`, but the two
dangerous scopes are removed unconditionally, so production builds can
never steer or inject into the runtime.
