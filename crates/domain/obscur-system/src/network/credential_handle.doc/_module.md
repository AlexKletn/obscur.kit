An opaque, host-resolved reference to a credential — never the secret itself
(spec §5.15, §7.12).

When a network fetch needs authentication, the kit must never hold the raw
secret: sensitive material stays with the host (privacy §7.12). A
[`CredentialHandle`] carries only an authentication *scheme* and an opaque
*reference* (a host-side handle/id) that the host — and only the host —
resolves to the actual secret when it fulfils the request (spec §5.15).
