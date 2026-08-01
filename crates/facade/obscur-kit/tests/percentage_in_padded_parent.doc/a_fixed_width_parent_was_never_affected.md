The same shape with a **fixed** parent width, which was always correct. It is
here because it is the reason the bug survived: whichever basis a `px` width
is resolved against it answers the same, so only percentages ever diverged.
