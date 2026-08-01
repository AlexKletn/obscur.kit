An opaque reference to a credential the host resolves, **never** the raw
secret (spec §5.15, §7.12).

The `reference` is not a password, token, or key: it is an opaque handle/id
the host maps to the real secret on its own side when performing a fetch.
The secret never enters the pure domain, honoring the privacy rule that
sensitive material stays with the host (§7.12). Even so the reference is
treated as sensitive — use [`Self::redacted_debug`] for logging so the
handle never leaks.
