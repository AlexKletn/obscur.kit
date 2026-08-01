The phase a listener *registers* for (spec §2.3).

§2.3's propagation is *"1. capture; 2. target; 3. bubble; 4. default action"*,
but only two of those are a **registration** choice: a listener asks to be run
on the way down or on the way up. [`EventPhase::Target`] is not a choice —
every listener attached to the target node runs when the event reaches it,
whichever direction it registered for — and the default action is not a
listener at all. Hence two variants, not four.

# Why the default is `Bubble`

Capture is the unusual request: it lets an ancestor see an event *before* the
node the user actually interacted with, which is how an overlay or a
gesture arbiter takes an interaction away from its target. Bubble is what a
handler ordinarily wants — "tell me after the thing that was hit has had its
say" — so it is the default, and capture must be asked for explicitly.
