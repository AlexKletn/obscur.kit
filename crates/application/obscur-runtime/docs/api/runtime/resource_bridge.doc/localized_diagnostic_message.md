The localized human message for a diagnostic `code`, in the language named by
`lang` (a BCP-47 language subtag such as `en`, `ru`, or `und`).

This is a **scoped** diagnostic-message catalog (spec §1.6, §7.2), not the
deferred number/date i18n formatting system: it maps a stable diagnostic
*code* to a per-language human string, so the machine-readable code stays
locale-invariant while the wording follows the user's locale. The lookup is
keyed first by the code (the fault class) and then by the language subtag,
and falls back to English for any language without its own entry — including
the undetermined `und` locale a host never localized.
