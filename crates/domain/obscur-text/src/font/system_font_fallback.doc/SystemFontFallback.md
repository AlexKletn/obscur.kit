Finds a face on the host that can draw a character the registered fonts
cannot (spec §5.10).

# Why a port rather than a bigger font set

An application's font chain is the handful of families it names plus the
generic ones a host installs — three faces, in this kit's default wiring.
Between them they cover Latin, Cyrillic and Greek and essentially nothing
else, so a CJK file name, an emoji, or a `⌫` on a key draws **nothing at
all**: a glyph that resolves to `.notdef` is skipped rather than boxed, which
reads as a broken control rather than as a missing font.

Registering every font on the machine up front is the obvious alternative and
the wrong one: a full CJK face is tens of megabytes, and an application pays
for all of them to render one character it may never see. This port asks the
question the other way round — *which face covers this character* — so the
cost is one lookup and one face, at the moment the character first appears.

# What an implementation owes

A **covering** face or [`None`]. Returning a face that does not actually
contain the character is worse than returning nothing: the resolver trusts
this answer and will register it, so a non-covering face becomes a permanent
wrong answer for that character. Implementations are consulted rarely (once
per new character, then cached by the caller) and may therefore be slow —
shelling out to a system tool is expected.
