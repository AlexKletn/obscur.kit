Returns the whitespace-normalized textual form of `ty`.

`ToTokens::to_token_stream().to_string()` already renders a single space
between tokens; this collapses any runs and trims the ends so equal types
hash identically regardless of the caller's source formatting.
