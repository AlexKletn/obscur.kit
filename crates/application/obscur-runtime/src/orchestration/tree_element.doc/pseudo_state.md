Reports the node's active pseudo-state: the union of its *interaction*
states (`:hover`/`:focus`/…, from the cursor's [`PseudoStates`] source, or
empty when the cursor is stateless / the node has no recorded state) with
its *structural* states (`:disabled`/`:checked`), read directly from the
node's own [`StyleAttributes`](obscur_view::view::StyleAttributes).

The structural bits are intrinsic to the element, so they are reported on
**both** the stateful ([`with_pseudo`](Self::with_pseudo)) and the
stateless ([`new`](Self::new)) paths — a `:disabled`/`:checked` rule bites
even without a live interaction map.
