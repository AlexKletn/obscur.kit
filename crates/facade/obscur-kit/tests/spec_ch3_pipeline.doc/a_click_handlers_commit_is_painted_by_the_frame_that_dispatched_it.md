§3.4 Conceptual pipeline — "3. dispatch input/events;" then "4. commit
state/tree transaction;" then "6. resolve styles;".

A click handler that commits `class active` must reach paint in the **same**
frame that dispatched the click: one frame, red text.
