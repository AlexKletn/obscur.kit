A click on a virtualized row resolves to the entry's **model index**.

A virtual row cannot carry a per-row `on:click`: every row a builder emits lowers
to the same command symbol, so a symbol can never say which row was hit. The
runtime resolves it from geometry instead — the container box, its scroll offset,
the row extent — which is the same "only the runtime holds both halves" argument
the virtual list itself rests on. These tests pin that the resolved index is the
row under the cursor, and that it tracks the scroll offset rather than the screen
position.
