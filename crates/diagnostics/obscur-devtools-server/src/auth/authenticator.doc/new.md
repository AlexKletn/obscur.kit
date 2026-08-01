Creates an empty authenticator that rejects every token.

This is the disabled-by-default state (spec §7.7); the host must
[`register_token`](Self::register_token) before any session can connect.
