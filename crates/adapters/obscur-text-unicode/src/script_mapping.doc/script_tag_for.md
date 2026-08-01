Resolves the itemization script of `ch`, or `None` when it is script-neutral.

Returns `Some(tag)` for a character that belongs to a concrete script,
mapping the handful of scripts the domain names explicitly to their
[`ScriptTag`] constant and every other assigned script to
[`ScriptTag::UNKNOWN`]. Returns `None` for characters whose script is
`Common` (spaces, punctuation, digits) or `Inherited` (combining marks):
these are *neutral* and must attach to the surrounding run rather than start
a new one, so the itemizer treats an absent tag as "extend the current run".
