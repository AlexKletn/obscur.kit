The one command symbol both rows bind — the same in the markup (`on:click=pick`)
and in Rust, because the front end hashes a handler name with the same FNV-1a
[`InternedSymbol::named`] uses.
