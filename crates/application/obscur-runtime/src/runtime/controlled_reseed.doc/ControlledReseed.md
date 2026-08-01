Whether an **external** rewrite of a focused field's retained value should
replace what the user is typing (WHATWG HTML §4.10.5.5: a text control
reflects its `value`, so when the app rewrites that value the control displays
the new value).

A pure decision over the retained tree and the live session: it reads, it
never writes. Rule 5 of this crate's decomposition keeps the write in the
runtime, because reseeding a session also means re-deriving the caret-follow
scroll, and rule 2 forbids this entity from holding what that is measured
against.

# Why the gate is an entity of its own

Because it is the one place the toolkit decides **who owns the text** — and
both wrong answers are silent. Reseed too eagerly and every keystroke is
clobbered by the tree the frame before it produced, so a fast typist watches
characters disappear. Reseed too rarely and a controlled field drifts from the
model it is supposed to mirror, so a `Clear` button leaves the old text on
screen. Neither shows up as an error anywhere; each is a rule about three
strings, and the rule is only correct when all three are compared in one
place.

That is also why the retained value is read here rather than handed in: the
discriminator is *which* of the three strings differ from which, and a caller
that fetched two of them itself would be one line from asking the wrong
question.
