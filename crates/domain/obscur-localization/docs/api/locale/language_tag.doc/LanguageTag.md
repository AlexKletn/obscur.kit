A BCP-47 language tag: the *current locale* of spec §7.2.

The supported shape is `language[-script][-region]` — `ru`, `ru-RU`,
`sr-Latn-RS`, `az-Arab-IR`. That is the subset every message catalogue and
every plural rule set is keyed by; variants, extensions and private-use
subtags (`de-DE-1996`, `en-US-u-ca-buddhist`, `x-pig-latin`) are **rejected**
by [`parse`](Self::parse) rather than silently dropped, because dropping them
would answer a question that was not asked.

Subtags are canonicalised on parse — language lower-case, script title-case,
region upper-case — so `RU-ru` and `ru-RU` are the same tag and compare equal.
The separator is `-`; a POSIX-flavoured `ru_RU.UTF-8` is an *operating system*
spelling, and normalising it is the job of whatever adapter read it off the
host, not of this domain type.
