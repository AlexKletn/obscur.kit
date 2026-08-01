A character outside the application's font chain is drawn, not dropped
(spec §5.10 font fallback).

The defect: an application's chain is the families it names plus the three
generics a host installs — Latin, Cyrillic and Greek between them. Anything
else (CJK, emoji, `⌫` on a key) resolved to no face, and an unresolved run
paints **no glyphs at all** — a blank where a character should be, which
reads as a broken control rather than as a missing font.

These tests need a host with fontconfig and real system fonts. Where there is
none they assert nothing, because there is nothing to ask — which is stated
here rather than hidden, since a silently-skipping test is worse than none.
