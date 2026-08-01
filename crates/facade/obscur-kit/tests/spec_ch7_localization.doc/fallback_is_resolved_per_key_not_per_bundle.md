§7.2: "Localization abstraction includes:" / "- current locale;" /
"- fallback;".

WEAKENED 2026-07-19: this originally also asserted that key `C`, absent from
both `ru-RU` and `ru`, resolves to `"C-en"` from a DEFAULT LOCALE `en`. §7.2
lists "- fallback;" as a bare bullet with no default-locale layer beneath it,
the case never configured one, and `Localizer` has no API that could (no
`set_default_locale`/`with_fallback_chain` — only `set_locale` and
`set_provider`). The two assertions that make the case's own point — that
fallback is decided PER KEY, so one format pass can draw `A` from `ru-RU` and
`B` from `ru` rather than committing to a single bundle — are kept, and still
fail an implementation that picks one bundle and stays in it. Candidate spec
amendment: state whether the localizer has a configurable default/root locale
beneath the current locale's own chain, and what a key missing from the whole
chain returns (the key itself, empty, or a diagnostic).
