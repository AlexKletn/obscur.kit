Formats the message `key` with `args` in the current locale (spec §7.2).

Fallback is [`LanguageTag::fallback_chain`]: `ru-RU`, then `ru`, then
`und`. The first locale the provider has a pattern for wins, and it is
formatted under the **current locale's** plural rules — not the matched
locale's, which are the same rules in every case the chain can produce and
would be a lie in none.

# A missing key renders as itself

Not as English, and not as empty. `format("file.count", …)` with nothing
registered returns `"file.count"` — visible on screen, greppable in a
screenshot, and impossible to mistake for a translation. A toolkit that
fell back to an English default string would make English the schema, which
is the thing §7.2 exists to prevent.
