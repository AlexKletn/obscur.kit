Creates a handle for authentication `scheme` (e.g. `"Bearer"`,
`"Basic"`) referring to the opaque host-side `reference` (spec §5.15).

`reference` must be an opaque handle the host resolves — never the raw
secret (§7.12).
