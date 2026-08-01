Resolves a host pointer event to the [`PointerLanding`] the rest of the
dispatch reads (spec §2.4, CSS Overflow 3 §scroll-container).

A **borrow bundle** over the retained frame — surface, tree, computed styles,
arranged boxes, live scroll offsets and the z-order index — held for one event
and released with it. It reads; it moves nothing and schedules nothing.

# Why the mapping and the targeting are one entity

Because targeting is not one lookup, it is a *precedence* between two answers
that are computed in two different coordinate spaces, and the space is what
makes each answer right. The capture answer needs no point at all; the
hit-test answer needs both of them, one for containment and one for clipping.
Split the mapping from the targeting and the caller has to remember which
point it holds and which the resolver wanted — which is exactly the mistake
that puts a click on the wrong row and leaves no trace anywhere.
