Formats the message `key` with `args` in the current locale (spec §7.2).

Fallback is [`LanguageTag::fallback_chain`]: `ru-RU`, then `ru`, then
`und`. The first locale the provider has a pattern for wins, and it is
formatted using the current locale's plural rules and number formatting.
Numeric arguments and plural `#` use localized digits and separators, with
the original number retained for plural and exact-match selection. The
original spelling is displayed if number formatting is unavailable.

# A missing key renders as itself

Not as English, and not as empty. `format("file.count", …)` with nothing
registered returns `"file.count"` — visible on screen, greppable in a
screenshot, and impossible to mistake for a translation. A toolkit that
fell back to an English default string would make English the schema, which
is the thing §7.2 exists to prevent.
