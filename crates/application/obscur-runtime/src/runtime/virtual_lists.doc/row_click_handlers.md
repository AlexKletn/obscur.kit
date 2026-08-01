A click handler per virtual list, run when a trusted click lands on one of its
rows with the resolved model index and the click count (1 single, 2 double).
This is how a virtualized row becomes clickable at all: every row realized from
one builder shares a single command symbol, so a symbol cannot say *which* row
was hit — only the geometry can, and only the runtime has it.
