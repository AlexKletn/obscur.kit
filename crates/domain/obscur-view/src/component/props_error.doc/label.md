The property's name as it is written in the component's own source — the field
name the `#[derive(Contract)]` took it from.

It travels beside the symbol because a symbol cannot be read back: an
[`InternedSymbol`](obscur_identity::InternedSymbol) is an FNV hash with no
reverse table, so a diagnostic that carried only the number would name the
property as `#2147483911` and leave the reader nothing to search for. The
literal is free — the derive already holds it — and it is the difference
between a two-minute fix and an afternoon.

Empty when the caller decodes by bare symbol and has no literal to give; the
message then falls back to the number.
