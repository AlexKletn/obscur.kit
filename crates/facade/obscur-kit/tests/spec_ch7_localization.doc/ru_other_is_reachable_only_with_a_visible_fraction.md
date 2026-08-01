§7.2: "- Russian plural categories;".

`specs/cldr/plurals.xml` line 190, verbatim:
`<pluralRule count="other">   @decimal 0.0~1.5, 10.0, 100.0, 1000.0, 10000.0,
100000.0, 1000000.0, …</pluralRule>`, read against the `v = 0` guard on every
other `ru` rule.
