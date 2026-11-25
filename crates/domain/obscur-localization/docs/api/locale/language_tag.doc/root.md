The undetermined root locale, `und`.

This is CLDR's `root`/`und`: the last link of every
[`fallback_chain`](Self::fallback_chain) and the locale a
[`Localizer`](crate::catalog::Localizer) resolves against when it has been
told nothing. It is not "English" — that mistake is the whole reason §7.2
names Russian in its minimum.
