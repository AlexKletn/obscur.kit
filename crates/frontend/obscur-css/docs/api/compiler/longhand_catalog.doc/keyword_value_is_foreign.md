Whether `values` is a bare keyword `property` cannot use — a foreign value
the lenient keyword path lowered to an inert interned keyword the property
ignores, i.e. an unparseable value that was dropped (§4.13).

Only the unambiguous cases are reported, so a clean sheet never trips it:
a `<color>` longhand handed a bare keyword that is not a named colour, or a
`<length>` longhand handed a keyword that is neither `auto` nor an intrinsic
sizing keyword. A CSS-wide keyword (`inherit`/`initial`/`unset`) is always
valid, and any non-keyword component (length, number, colour, function) is
judged by ordinary lowering, never here.
