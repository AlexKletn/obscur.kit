A virtual list builds a screenful of nodes while keeping the full extent.

Both halves matter, and only the runtime can hold them at once. Building just the
window is what makes a 10,000-entry directory affordable; stating the true content
height is what a scroll container needs to size a scrollbar. An app can do the
first for itself — it cannot do the second, because to state the full height it
would have to build the full list, which is the cost being avoided.
