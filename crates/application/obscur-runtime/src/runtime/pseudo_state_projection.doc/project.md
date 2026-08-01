Builds the per-node [`PseudoStates`] from the current hover chain, focus
and outstanding presses: [`PseudoState::HOVER`] on every hovered node,
with [`PseudoState::FOCUS`] merged into the focused node.
