Dispatches the §2.3 pair for a move that has already happened: `blur` on the
node that lost focus, then `focus` on the node that gained it.

Either half may be absent, and the two absences are the two ends of the same
transition — focusing the first control of a session blurs nothing, and a press
on empty space clears focus and focuses nothing. Both cases fire the half that
has a node and nothing else; neither is silent.
