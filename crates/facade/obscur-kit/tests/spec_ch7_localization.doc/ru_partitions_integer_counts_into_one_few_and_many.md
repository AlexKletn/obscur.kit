§7.2: "Default implementation should be non-Anglocentric and support, at
minimum:" / "- Russian plural categories;".

`specs/cldr/plurals.xml` line 187, verbatim:
`<pluralRule count="one">v = 0 and i % 10 = 1 and i % 100 != 11 @integer 1,
21, 31, 41, 51, 61, 71, 81, 101, 1001, …</pluralRule>` and line 189
`<pluralRule count="many">v = 0 and i % 10 = 0 or v = 0 and i % 10 = 5..9 or
v = 0 and i % 100 = 11..14 @integer 0, 5~19, 100, 1000, …</pluralRule>`.
