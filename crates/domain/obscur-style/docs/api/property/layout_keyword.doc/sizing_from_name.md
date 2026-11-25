Maps an intrinsic sizing keyword string to its reserved [`InternedSymbol`].

`auto` is deliberately **not** here: it is a length keyword
([`Length::Auto`](crate::property::Length::Auto)), not a sizing question, and
the CSS layer lowers it before reaching this table. Adding it here would
re-route `width: auto` through the keyword path and change its computed value.
