`PluralRules`: a locale's cardinal plural rule set, transcribed from CLDR
(spec §7.2).

Every rule function here is a line-by-line transcription of one
`<pluralRules locales="…">` element of `specs/cldr/plurals.xml` (CLDR
release-48, vendored). Each carries its CLDR source verbatim in its doc
comment and is tested against that element's own `@integer` sample list, so
the transcription is checkable by `grep` and by test rather than by trust.
