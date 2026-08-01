Returns the current locale's language subtag — the BCP-47 hint the shaper
takes (spec §7.2 text shaping integration).

A shaper wants the language, not the full tag: it decides
language-sensitive OpenType features (`locl`) and, through it, whether Serbian
Cyrillic gets its own italic **б**. The region is not its business.
