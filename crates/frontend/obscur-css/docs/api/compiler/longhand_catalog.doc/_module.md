The catalogue of modelled longhands: which authored names are longhands, and
the three type questions the compiler asks about one (§4.13, §4.14).

[`property_id_from_name`] is the crate's only name → [`PropertyId`] mapping —
a table that answers one question about every input, which is why it is an
entity rather than a helper. The three predicates beside it are the same
table read by type instead of by name: whether a longhand takes a `<length>`
(so a unitless `0` is `0px`), whether it takes a `<color>` (so a bare keyword
can never be valid), and which closed keyword vocabulary it draws on
([`layout_keyword`]'s reserved cross-crate symbols). The compiler's dispatch
consults all four; none of them lowers a value.
