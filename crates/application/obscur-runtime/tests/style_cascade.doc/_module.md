Authored style becomes computed style becomes paint (spec §4.14).

Six tests that follow one value from a stylesheet to a display list. Three of
them are about **origin**: a UA default sheet at `SystemDefault` supplies a base
look, an author sheet at `GlobalApp` overrules it, and with no default sheet
installed there is nothing to inherit but the initial value. That is precedence
by origin, not by specificity, which is why the fixture styles a `button` *type*
rather than a class — a class would let specificity confound the answer.

The other three are about **state**: a structural `:disabled` must bite only
while the element reports the state and re-resolve when it stops, and a recorded
`:hover` must reach paint and stop reaching it when the state is cleared. A rule
that never stops applying is the failure worth guarding, and it looks exactly
like a passing test until the state goes away.
