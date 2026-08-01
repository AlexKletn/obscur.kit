`Card`'s typed props. The `#[contract(name = "Card")]` override is what ties
the Rust type to the authored tag — the derive hashes *that* string into the
component id, and so does the front end when it meets `<Card/>`.
