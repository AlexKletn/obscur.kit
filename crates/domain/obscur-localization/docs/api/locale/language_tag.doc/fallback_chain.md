Returns the lookup fallback chain, most specific first, always ending at
[`root`](Self::root) (spec §7.2 fallback).

This is BCP-47 lookup: truncate one subtag from the right at a time.
`sr-Latn-RS` → `sr-Latn` → `sr` → `und`. A resolver walks it in order and
takes the first hit, so a catalogue that translates `sr` but not
`sr-Latn-RS` still answers, and one that translates neither falls through
to `und` rather than to English.

The chain always contains `self` first and `und` last; for `und` itself it
is one element long.
