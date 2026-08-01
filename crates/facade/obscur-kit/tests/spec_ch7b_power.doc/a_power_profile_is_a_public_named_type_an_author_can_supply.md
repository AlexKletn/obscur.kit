§7.11, verbatim: "Optional `PowerProfile` may influence: background work;
visual effects; telemetry; polling."

The absent-surface probe. An author reading §7.11 writes the identifier the
spec spells in code font and hands the value to the system-provider layer at
startup.

Scope note: the second half of the case — "there is a public way to
install/supply it to a running app" — is **not expressible** here. No
runtime, host or bootstrap entry point in the public API accepts a
`PowerProfile` or a `PowerProvider`, so there is no call to write. That
absence is reported as a finding rather than as a non-compiling test; what
this case can still establish is that the *name* exists and that a profile
can be supplied to a provider.
