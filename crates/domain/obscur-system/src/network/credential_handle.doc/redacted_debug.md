Returns a log-safe description that names the scheme but **redacts** the
reference, e.g. `"CredentialHandle(Bearer, <redacted>)"` (spec §7.12).

The reference is opaque but still treated as sensitive, so it is never
emitted here; use this instead of the [`Debug`] representation when
logging so the handle can never leak.
