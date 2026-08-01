Renders every component whose output is stale and reconciles it into that
node's children, returning whether any of those sub-reconciles changed the
tree.

# Only the stale ones

A component is stale when it has just mounted (nothing has rendered it),
when its node's authored props changed (its typed props may decode
differently), or when it asked to be re-rendered via
[`ComponentContext::request_update`]. Everything else keeps the output it
already has — this is what makes a component's state change cost its own
subtree rather than the document (spec §3.1: "dependency tracking
invalidates only consumers of changed values").

Each render is enacted as a [`UiOperation::Rerender`] through the same
transaction boundary a root reconcile uses, so a component's output is
committed and versioned exactly like the root's.

A component's props are decoded from the node's **retained** properties —
the desired tree is already consumed by the time this runs, which is why
the reconciler retains them.

# Why a fixpoint loop

A component's output may itself contain component-backed elements, which
only exist (and so only mount) once that output is reconciled. Each pass
therefore renders the components that appeared in the previous one, until a
pass produces no new stale instances. Each component renders **at most
once** per frame: a request raised during its own render is honoured by the
next frame, so a component that always asks for an update cannot spin this
loop. [`MAX_PASSES`] additionally bounds mutual nesting, so a
self-rendering pair truncates the frame rather than hanging it.

# A component whose render fails

Its error boundary answers for it (spec §1.7), and the answer is enacted
here: a retry leaves the subtree for the next frame, an exhausted budget
commits the component's fallback UI *over* the subtree, and a disabled
boundary hands the failure to the nearest enclosing enabled one — whose
own subtree is then replaced. Every one of those is reported as a
[`Diagnostic`](obscur_diagnostics::Diagnostic) naming the node and the
recovery action taken (spec §1.6), on top of the `component.props_errors`
counter.

# Where the previous output still stands

Two cases leave the failed subtree showing what it last rendered, and
neither is silent:

- a **retry** keeps it for exactly the frames the policy's budget buys,
  then replaces it. The component stays stale, so the frame end asks the
  scheduler for the next one and the budget actually gets spent.
- a **propagated** failure with no enclosing enabled boundary
  ([`contain_escaped`](Self::contain_escaped) finding none) leaves it
  indefinitely: every boundary that could have answered was disabled, so
  the runtime reports the failure and recovers nothing.
