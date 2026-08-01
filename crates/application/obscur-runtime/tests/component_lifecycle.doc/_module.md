Mounting, re-rendering and unmounting a component (spec §2.1, §2.14, §3.1).

The fixture is a counter that adds its `step` prop to its own local state and
emits the running total, which makes the rendered text an oracle for three
different questions at once: did the props decode, did `render` run *this*
frame, and did the instance survive. "2 + 3 = 5" is the state persisting; "2"
where "4" was possible is a render that correctly did **not** happen.

The vertical slice comes first — an element naming a registered kind mounts an
instance, its typed props decode from the authored properties, and its output
becomes that node's children. Then the two halves of incremental invalidation:
of two sibling counters, only the one whose props changed re-renders, and an
identical frame re-renders neither. Both are stated as "the total did not
advance", which is the only way to observe a render that was skipped.

Reconciliation is the subtle one. The desired node `<Counter step=N/>` has no
children — the component supplies them — so a root reconcile that took the
desired children as authoritative would delete the component's output every
frame and re-create it with fresh ids, silently remounting anything nested
inside. The `COMPONENT_HOST` flag is what stops that, and the test asserts on
the *identity* of the output node, which is the only thing a re-creation cannot
fake. Its counterpart: unmounting clears the flag, or the node would stay opaque
to reconciliation forever.

The last is the boundary seen from the component's side: props that stop
decoding buy exactly one more frame from the default budget, and then the last
good output stops standing in for the current one. What the boundary *reports*
while doing that is `tests/error_boundaries.rs`.
