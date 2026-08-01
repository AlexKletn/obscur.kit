A [`Hyphenator`] backed by the embedded US-English Knuth-Liang patterns
(spec §5.10).

[`EnglishHyphenator::new`] loads the `hyphenation` crate's embedded
`Language::EnglishUS` dictionary once; [`hyphenate`](Hyphenator::hyphenate)
then asks it for a word's break opportunities and returns their byte offsets.
The dictionary's built-in minimum prefix/suffix bounds keep a break from
stranding a single letter, and the standard patterns are deterministic, so
the same word always yields the same offsets.
