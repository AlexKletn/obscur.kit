Returns `true` for the **out-of-flow** schemes
([`Absolute`](Self::Absolute) / [`Fixed`](Self::Fixed)).

An out-of-flow node is excluded from its parent's flow/flex/grid
algorithm entirely: it neither contributes to the parent's content size
nor shifts its in-flow siblings, and is positioned separately against its
containing block.
