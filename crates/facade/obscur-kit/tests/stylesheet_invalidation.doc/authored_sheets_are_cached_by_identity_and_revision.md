The §4.12 cache key, which the authored set never had: sheets are handed over as
*identified* sources, so they are cached at all, and a replacement replaces an
entry instead of piling up a new one. An anonymous source — what `App` used to
build — is deliberately never cached, because two unrelated sheets would collide
under one key.
