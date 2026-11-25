A 100×100 `panel` holding two stacked 100×50 children, `a` over `b`, each bound
to its own click command.

The geometry is the whole point and every test that uses it depends on it: `a`
occupies y ∈ [0, 50) and `b` occupies y ∈ [50, 100), so a press at y = 25 and a
press at y = 75 are unambiguously two different controls. Two commands rather
than one is what lets a release be *attributed* — the failure this fixture was
built to expose is a second contact overwriting the first, where one of the two
buttons simply never fires and which one depends on the order the fingers landed.

The associated functions serve tests that render the panel themselves:
[`view`](Self::view) for the unbound panel, [`command_view`](Self::command_view)
for arbitrary commands, [`ancestor_bound_view`](Self::ancestor_bound_view) for
the arrangement where a click has to bubble, [`sheet`](Self::sheet) for the sheet
that sizes them, and [`nodes`](Self::nodes) for the three ids after a frame.
