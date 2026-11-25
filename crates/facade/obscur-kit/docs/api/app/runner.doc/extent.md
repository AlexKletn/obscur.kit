The surface's size in physical pixels, tracked here.

The surface itself is on the acquire thread, so it cannot be asked. Kept in step
with it by the resize path, and carried alongside each acquired image so a blit
cannot use the wrong one.
