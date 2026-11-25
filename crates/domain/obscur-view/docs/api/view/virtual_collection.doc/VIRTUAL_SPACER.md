A realized virtual list's spacer: an empty box standing in for the rows outside
the window, sized to exactly the space they would have taken.

It is what makes the container's content extent true without building the rows,
so the scrollbar is proportioned correctly and native scrolling covers the whole
list rather than only the part that happens to exist.
