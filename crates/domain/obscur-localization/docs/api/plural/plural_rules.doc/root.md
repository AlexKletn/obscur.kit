CLDR `<pluralRules locales="bm bo dz hnj id ig ii in ja jbo jv jw kde kea km
ko lkt lo ms my nqo osa root sah ses sg su th to tpi vi wo yo yue zh">`:

```text
other:  @integer 0~15, 100, 1000, 10000, 100000, 1000000, …
```

One category.

CLDR lists the undetermined locale as `root` in this element's `locales`
attribute. It is absent from [`RULE_SETS`] and handled by
[`PluralRules::cardinal`]'s tail instead, because the root is where *every*
fallback chain ends: registering it as a match would make a locale nobody
transcribed report [`SupportLevel::Native`] on its way past.
