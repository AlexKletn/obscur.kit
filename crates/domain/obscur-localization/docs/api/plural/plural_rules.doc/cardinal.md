Returns the cardinal rules for `locale`, walking its
[`fallback_chain`](LanguageTag::fallback_chain).

Lookup is by the chain, not by the language alone, so a locale whose rules
differ from its language's (CLDR spells one such group `pt_PT`) can be
registered and found. The first link with a registered rule set wins.

# Which locales are transcribed

The six `<pluralRules>` groups of `specs/cldr/plurals.xml` listed below —
chosen to span every distinct *shape* CLDR uses (1, 2, 4 and 6 categories;
rules keyed on `i`, on `n`, on `v`, and on `i % 100`), with Russian's
four-category rule the one §7.2 names:

- **root** and 34 languages including `ja`, `zh`, `ko`, `th`, `vi`, `id` —
  one category;
- 18 languages including `en`, `de`, `nl`, `sv`, `fi` — two;
- `ru`, `uk` — four;
- `pl` — four, and *not* Russian's four;
- `cs`, `sk` — four, keyed differently again;
- `ar`, `ars` — all six.

Everything else — including French, Spanish, Portuguese and Italian, whose
CLDR rules are written in terms of the compact-decimal exponent operand
that [`PluralOperands`] does not model — is **not** transcribed and reports
[`SupportLevel::Unavailable`]. Adding a group is one function, its CLDR
line, its sample list, and a row in [`RULE_SETS`].
