Closes the region and returns whether the painter left it balanced.

`false` means the painter left a scope open or attempted an invalid pop.
Closing the region restores the enclosing scope so later drawing can proceed.
The host can use the result to report the fault.
