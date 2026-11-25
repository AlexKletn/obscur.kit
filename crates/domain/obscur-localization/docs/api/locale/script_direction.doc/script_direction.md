Returns the base [`TextDirection`] for an ISO-15924 `script` code.

The comparison is exact and case-sensitive against CLDR's canonical
title-case spelling (`Arab`, not `arab`); callers hand this the already
canonicalised subtag that
[`LanguageTag::parse`](crate::locale::LanguageTag::parse) produced.
