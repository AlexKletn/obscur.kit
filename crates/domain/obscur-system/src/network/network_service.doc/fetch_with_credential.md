Fetches `request`, authenticating with `credential` (spec §5.15).

The default implementation simply delegates to [`Self::fetch`]: the
[`CredentialHandle`] carries only an opaque reference (§7.12), so the
host is responsible for resolving it to the real secret and injecting
the actual `Authorization` header when it performs the fetch — the kit
never holds the secret. An adapter may override this to thread the
credential through its own authentication flow.

# Errors

Returns a [`NetworkError`] when the underlying fetch cannot be satisfied.
