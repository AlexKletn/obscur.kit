Loads the embedded US-English dictionary.

The dictionary is compiled into the binary by the `hyphenation` crate's
`embed_en-us` feature, so loading it is infallible in practice.

# Panics

Panics if the embedded dictionary cannot be deserialized, which indicates
a broken build rather than a runtime condition.
