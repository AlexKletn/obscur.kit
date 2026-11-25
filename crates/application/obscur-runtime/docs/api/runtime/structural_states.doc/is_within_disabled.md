Whether `node` lies within a **disabled** control — the node itself or any
of its element ancestors carries the reserved `:disabled` structural state.

A disabled control and its whole subtree are inert to interaction: a press
seats no focus and no editing caret, typing never reaches it, and a click /
Enter / Space fires no activation command — matching a native disabled form
control. `:hover` is deliberately unaffected (CSS still hovers a disabled
element), so only the *behavioural* paths gate on this, not styling.
