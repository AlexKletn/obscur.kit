Reports every slot name this component uses that its own contract does not
declare, on either side of the projection.

The check has exactly one place it can run: here, after the component rendered
and before [`project_slots`], because projection substitutes every outlet away
and the render output stops being observable. It needs both halves — the output
holds the outlets, the retained host node holds the authored content — and it
needs the contract, which only the registry has.

Severity is `Error` and no recovery is claimed, the way the stray-outlet report
in `ReconcilePhase` — the same class of fault — already does it. Nothing stops
and the frame is still committed; the reported content or outlet is exactly what
projection is about to drop. That is the honest pairing: a diagnostic that names
what was lost, rather than a failure that loses the frame as well.

Free for the components that declare no slots and are given no children, which
is every one that does not take content: [`SlotMismatch::find`] walks the output
only for outlets and returns immediately on an empty result.
