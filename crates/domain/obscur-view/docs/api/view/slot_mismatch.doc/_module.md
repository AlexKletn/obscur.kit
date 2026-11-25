The §2.12 check nobody was doing: what a component's contract *declares* against
what its render and its author actually *use*.

Slot projection matches by name and drops what does not match. That is the right
runtime behaviour — an unfilled outlet must not become an empty box, and content
naming a slot that is not there has nowhere to go — but on its own it means a
single typo is invisible. A component rendering `<slot name="hedaer"/>` received
nothing, forever, in silence; an author writing `<div slot="hedaer"/>` had their
markup dropped just as quietly.

The contract is the thing that makes both checkable, because it is *static*: it
says which slots exist regardless of what this frame happened to render. So a
conditional outlet — one a component renders only when a prop is set — is not a
mismatch, and neither is a slot the author left empty. Only a name that appears
on one side and exists on neither is.

[`SlotMismatch::find`] is pure and does not know what a diagnostic is; the
runtime phrases the report. That keeps this in the domain, where the front ends
can reach it too when they can see a contract.
