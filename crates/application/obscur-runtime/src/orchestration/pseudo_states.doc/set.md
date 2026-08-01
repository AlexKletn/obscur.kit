Records `state` as `node`'s interaction state, replacing any previous
entry.

Setting [`PseudoState::empty`] removes the node's entry, keeping the map
canonical: an absent node and a node explicitly set to empty are
indistinguishable to [`get`](Self::get).
