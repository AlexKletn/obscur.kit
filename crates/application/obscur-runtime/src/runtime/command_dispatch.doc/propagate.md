Runs §2.3's steps 1–3 — capture, target, bubble — over the node path from
the root down to `origin`, returning how many listeners ran.

The path itself is computed by the **domain**
[`EventDispatcher`](obscur_interaction::event::EventDispatcher) rather than
here: it is the type that already knows a path is "a `Capture` step per
ancestor, then `Target`, then a `Bubble` step per ancestor in reverse", and
it is already covered by the §2.3 spec test. Until this method existed,
nothing in the workspace called it — the model was exported and driven by
nobody.

At every step [`UiEvent::current_target`] moves to the node being visited
while [`UiEvent::target`] stays at `origin`; that pair is what lets an
ancestor's listener tell "what was hit" from "what I am attached to". The
walk stops as soon as [`UiEvent::is_stopped`] holds, so
`stop_propagation` on the target keeps an ancestor's bubble listener from
running at all.

With `origin = None` there is no node path — §2.6's window and application
links are not tree positions — so nothing propagates and only step 4 can
run.
