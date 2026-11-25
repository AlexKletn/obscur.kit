CLDR `<pluralRules locales="ast de en et fi fy gl ia ie io ji lij nl sc sv sw
ur yi">`:

```text
one:    i = 1 and v = 0   @integer 1
other:                    @integer 0, 2~16, 100, 1000, …
```

Two categories — and note that even *this* is not `n == 1`: `1.0` is `other`,
because `v = 1`.
