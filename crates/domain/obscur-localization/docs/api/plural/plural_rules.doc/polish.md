CLDR `<pluralRules locales="pl">`:

```text
one:    i = 1 and v = 0   @integer 1
few:    v = 0 and i % 10 = 2..4 and i % 100 != 12..14
          @integer 2~4, 22~24, 32~34, 42~44, 52~54, 62, 102, 1002, …
many:   v = 0 and i != 1 and i % 10 = 0..1 or v = 0 and i % 10 = 5..9 or v = 0 and i % 100 = 12..14
          @integer 0, 5~19, 100, 1000, 10000, 100000, 1000000, …
other:    @decimal 0.0~1.5, 10.0, 100.0, 1000.0, …
```

Four categories, like Russian's, and **not** Russian's: Polish `one` is 1
alone (21 is `many`, where Russian's is `one`), and its `many` excludes 11
from the `i % 100` clause that Russian includes. Sharing a rule function
between them because "both are Slavic and both have four" would be wrong for
every number ending in 1 except 1 itself.
