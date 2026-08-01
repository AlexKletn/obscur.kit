A [`ResourceDecoder`] that validates and admits font faces via `ttf-parser`
(spec §5.18).

This decoder supports only [`ResourceKind::Font`]. It parses the first face
(index 0) of the given bytes to confirm they form a usable face — malformed
bytes become [`ResourceError::DecodeFailed`] — then returns the validated
bytes unchanged as the [`DecodedResource`] CPU payload. Font collections are
admitted through their first face.
