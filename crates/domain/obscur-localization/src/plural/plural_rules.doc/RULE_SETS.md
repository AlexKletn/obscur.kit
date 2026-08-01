The registry of transcribed CLDR rule sets, keyed by the exact canonical
locale spellings of each `<pluralRules locales="…">` element.

Order is irrelevant: the groups are disjoint (CLDR lists each locale once).

# The root must never be registered here

CLDR lists `root` in the one-category group's `locales`, and it is left out on
purpose — enforced by `the_root_is_never_a_registered_rule_set`. Every
[`LanguageTag::fallback_chain`] *ends* at the root, so a root entry would be
found for every locale on earth and [`PluralRules::cardinal`] would report
[`SupportLevel::Native`] for all of them. The root's rules are the tail of
`cardinal` instead, where reaching them is correctly an admission that nothing
matched.
