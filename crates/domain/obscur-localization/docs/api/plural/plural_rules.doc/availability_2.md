Returns whether CLDR rules were found for the requested locale
(spec §7.2).

[`SupportLevel::Native`] means a transcribed rule set answered.
[`SupportLevel::Unavailable`] means none did and
[`select`](Self::select) is answering [`PluralCategory::Other`] for
everything — correct for `und`, and a *known* gap for anything else.
