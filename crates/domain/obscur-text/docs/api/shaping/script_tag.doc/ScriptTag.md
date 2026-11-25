An ISO-15924 script identifier, stored as four ASCII bytes (spec §5.10).

Itemization splits mixed-script text into runs of a single script so each can
be shaped with a covering font. A `ScriptTag` names that script with its
canonical ISO-15924 code — `Latn`, `Cyrl`, `Hani`, `Arab`, and so on — packed
into a `[u8; 4]`. Two tags of special interest are [`COMMON`](Self::COMMON)
(`Zyyy`, for punctuation and other script-neutral characters) and
[`UNKNOWN`](Self::UNKNOWN) (`Zzzz`, for unassigned code points).

This type stores the bytes verbatim; it does not validate them against the
registered ISO-15924 set.
