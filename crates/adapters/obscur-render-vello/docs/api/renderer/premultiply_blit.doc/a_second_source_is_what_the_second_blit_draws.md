A blit through a second source view draws the second source.

This is the test that catches a bind group cached past the view it was built
from. The two sources differ in nothing a cache could key on except identity —
same extent, same format, same descriptor — so a cache keyed on any of those, or
on nothing at all, keeps drawing the first source and fails the second read.

It needs a GPU: a bind group is a device resource and the claim is about what
reaches the target. So it is `#[ignore]`d — counted as ignored, never as a pass —
and fails outright if it is asked for where no adapter exists. See `gpu`.
