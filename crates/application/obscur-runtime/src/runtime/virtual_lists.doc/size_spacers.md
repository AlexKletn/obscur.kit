Gives each realized virtual-list spacer the height of the rows it stands in
for (spec §4.9).

The spacers are what make a virtualized container's extent true: the window's
rows are real nodes, and everything above and below them is one box sized to
exactly the space those items would occupy. Without this the scroll container
would measure only the window, so the scrollbar would claim the list is a
screenful long and scrolling would stop at the end of what happens to be
built. The size cannot come from CSS — it changes with the scroll position and
the model — so it is applied here, next to the replaced-element sizing that
has the same shape.
