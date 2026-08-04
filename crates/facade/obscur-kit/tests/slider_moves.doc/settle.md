Re-renders with the value the control reported, which is what an application does.

A slider is CONTROLLED: the runtime never writes into the tree. A test that
skipped this would be driving a slider whose value never moves, and every
gesture after the first would read the original.
