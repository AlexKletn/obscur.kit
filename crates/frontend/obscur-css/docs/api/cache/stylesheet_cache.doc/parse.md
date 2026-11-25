Returns `source`'s parse, parsing it only on a miss.

This is the whole point of the cache, so it is the method to reach for: a
caller that parses first and inserts second has already paid the cost the
cache exists to skip.
