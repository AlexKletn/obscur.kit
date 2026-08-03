Closes the region and returns whether the painter left it balanced.

`false` means the painter opened a bracket it never closed and this closed it
for them. The list is left balanced either way; the boolean is so a host can
report the fault instead of repairing it in silence every frame.
