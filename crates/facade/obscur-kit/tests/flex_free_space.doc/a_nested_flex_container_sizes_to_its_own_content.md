The case a real panel hit: the item is itself a **flex container**, which is
what every styled button in this kit is.

It needed a second fix. A flex container with `width: auto` reported the space
it was *offered* along its main axis — the block answer, "as much as I was
given" — so under the max-content probe it answered infinity, and the caller
fell back to the whole line. The answer has to be summed from its items, and
summed rather than read off their placed positions: `justify-content` has
already spread them across the infinite line, so the rightmost edge is
infinite too.
