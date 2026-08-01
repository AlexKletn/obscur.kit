CLDR `<pluralRules locales="cs sk">`:

```text
one:    i = 1 and v = 0   @integer 1
few:    i = 2..4 and v = 0   @integer 2~4
many:   v != 0   @decimal 0.0~1.5, 10.0, 100.0, …
other:    @integer 0, 5~19, 100, 1000, …
```

Four categories again, and different again — here `many` is the category of
*written fractions*, and the integers Russian calls `many` are `other`. Three
four-category Slavic languages, three incompatible rules.
