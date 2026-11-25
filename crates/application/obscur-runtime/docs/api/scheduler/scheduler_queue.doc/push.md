Inserts a request, coalescing by reason.

Returns `true` if the request was newly added, or `false` if an entry
with the same reason was already pending and this push was coalesced into
it.
