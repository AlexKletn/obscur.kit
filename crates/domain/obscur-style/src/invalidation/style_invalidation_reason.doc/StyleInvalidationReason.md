The typed reason a node's computed style was invalidated (§4.3).

Every entry in a [`crate::invalidation::StyleInvalidation`] set carries one of
these so the cause of a restyle is explicit rather than implied by a bare
dirty flag. `Copy + Eq + Hash` so it can be stored in maps/sets cheaply and
compared without allocation.
