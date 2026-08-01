Localization: keys, parameters, locale, fallback, and CLDR plural rules
(spec §7.2).

# What is here

§7.2's localization abstraction, assembled behind
[`Localizer`](catalog::Localizer): a **key** ([`MessageKey`](message::MessageKey))
and **parameters** ([`MessageArgs`](message::MessageArgs)) resolved against a
**current locale** ([`LanguageTag`](locale::LanguageTag)) through a
**replaceable provider** ([`MessageProvider`](catalog::MessageProvider)) with
**fallback** ([`fallback_chain`](locale::LanguageTag::fallback_chain)), plus
**RTL integration** ([`direction`](locale::LanguageTag::direction)) and **text
shaping integration** ([`language`](catalog::Localizer::language), the BCP-47
hint a [`ShapingRequest`](obscur_text::shaping::ShapingRequest) carries).

The non-Anglocentric minimum §7.2 names by name:

- **Russian plural categories** — [`PluralRules`](plural::PluralRules) over
  [`PluralOperands`](plural::PluralOperands), transcribed from CLDR. Russian's
  four categories key on the last *two* digits and on whether the number was
  written with a fraction: 21 is `one`, 111 is `many`, `1.5` is `other`.
  English's two are the special case.
- **Numeral agreement** — a `plural` selector in a
  [`MessagePattern`](message::MessagePattern), whose `#` interpolates the
  number into the agreeing arm.
- **Gender** and **case variants** — a `select` selector, keyed by
  [`GrammaticalGender`](message::GrammaticalGender) and
  [`GrammaticalCase`](message::GrammaticalCase). Both are open value spaces,
  because CLDR's are: Russian has three genders and Polish five.

Every rule and every table is transcribed or derived from pinned CLDR
release-48 sources vendored at `specs/cldr/`, and each cites its source line.

# What is not here, and where the boundary is

- **Locale-aware dates, numbers and currencies** — §7.2 names these too and
  they are **not built**. They need per-locale CLDR pattern and symbol data
  (`ru`'s `NBSP` group separator, its `₽` placement, its month genitive forms),
  which is a data-vendoring problem of a different order to the ~40 lines of
  rules this crate transcribes. [`MessageValue::integer`](message::MessageValue::integer)
  therefore renders ASCII digits with no grouping in every locale. The plural
  *category* is fully locale-aware; the digits are not.
- **The compact-decimal exponent operand** (`c`/`e`) — not modelled, which is
  why the CLDR rule sets that use it (French, Spanish, Portuguese, Italian) are
  deliberately absent rather than half-transcribed. See
  [`PluralOperands`](plural::PluralOperands).
- **Ordinal plurals** (`1st`/`2nd`) — CLDR's `<plurals type="ordinal">` is a
  second rule set; only cardinals are here.
- **Reading the host's locale** — that is an adapter's job (§1.1): nothing in
  this crate touches the environment, and a host tells the runtime its locale
  rather than being asked. No adapter ships one yet.
- **Translation management** — §7.2, verbatim: *"The toolkit is not a full
  application translation-management platform."* Extraction, merge and
  translator workflow live beyond
  [`MessageProvider`](catalog::MessageProvider).
