The one place §2.3's event model and §2.6's routing chain meet.

It is a **borrow bundle** over exactly what a dispatch touches — the retained
tree it walks and the registry it runs handlers out of — held for one dispatch
and released with it. Rule 2 of this crate's decomposition keeps the frame
scheduler out: the dispatch *reports* that something ran
([`CommandDispatchOutcome`]) and the runtime requests the frame, because a
phase that held the scheduler would be claiming to own the frame.

# Why propagation and routing are one entity

Because §2.6's chain **is** §2.3's step 4. The spec's four steps are capture,
target, bubble, default action, and running the node's bound handler is what a
click's default action *is* — so the chain is the remainder of the propagation
rather than a mechanism bolted beside it. Two observable consequences follow
from that single sentence, and neither is expressible if the two halves live
apart: [`UiEvent::prevent_default`] suppresses the handler while leaving the
listener walk running, and [`UiEvent::stop_propagation`] halts the walk *and*
the chain.

The interface is narrow — resolve a binding, build a route, dispatch — and the
internals are a bubbling search, a five-link ordered chain with an inversion
the spec asks for, and a phase-by-phase listener walk driven by a domain
[`EventDispatcher`].
