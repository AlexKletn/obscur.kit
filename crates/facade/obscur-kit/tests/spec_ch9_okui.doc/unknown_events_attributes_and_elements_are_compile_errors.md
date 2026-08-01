§2.13 Component contracts — "Unknown attributes, events and string method
names are compile errors." (guardrail §9.6 "introduce untyped string
dispatch")

NG-OKUI-UNKNOWN-01.

WEAKENED 2026-07-19: this also required the diagnostic for a mistyped event
to SUGGEST the declared one — `text.contains("click")` for `on:clik`. §2.13
is one sentence, "Unknown attributes, events and string method names are
compile errors.", and the engine meets it: it reports "unknown event 'clik'
on component 'Button'", naming both the mistake and the component. A
did-you-mean suggestion is a quality-of-implementation nicety the spec does
not ask for. Candidate spec amendment: if near-miss suggestions are wanted
(they are the difference between a five-second and a five-minute fix on a
contract with thirty props), say so, and say what the edit-distance bar is.

Also DROPPED, half (c): "an unregistered element is NOT silently a generic
box", which compiled `<div>x</div>` and required it to fail. Two things were
wrong. §2.13's sentence enumerates "Unknown attributes, events and string
method names" — ELEMENTS are conspicuously not in it. And the fixture picked
the wrong subject anyway: `div` is a registered builtin, so it compiles
correctly and the case would have been unsatisfiable even if the requirement
existed. The test name still says "elements"; it is left alone so this note
is what a reader finds. Candidate spec amendment: say whether an element tag
that resolves to no registered component is a compile error, and if so how a
document addresses the builtin set — silently lowering an unknown tag to a
generic box is exactly the untyped escape hatch §9.6 lists as a guardrail.
