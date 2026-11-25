Negotiates the highest version supported by both `self` (the serving
side) and the `client` range.

Returns the highest mutually supported version, or `None` when the two
ranges do not overlap (for example, a major-version mismatch).
