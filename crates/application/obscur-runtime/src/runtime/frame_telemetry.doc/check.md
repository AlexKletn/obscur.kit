Checks this phase's `metric` against **this phase's** bound in `budget`.

Private and taking the phase rather than being chosen by the caller: the
bound a count is checked against is a property of the phase, not of the
call site.
