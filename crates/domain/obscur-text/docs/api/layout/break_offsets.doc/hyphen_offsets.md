Collects the interior hyphenation offsets a `hyphenator` proposes across
the words of `text`, mapped into full-text byte coordinates (spec §5.10).

The text is split into the same words the greedy packer sees (the spans
between adjacent [`LineBreak::opportunities`], and the leading/trailing
spans); each word substring is offered to the hyphenator, and every
returned offset is shifted by the word's start. Offsets that are not
strictly interior to their word are dropped, so an over-eager adapter can
never split at a word edge.
