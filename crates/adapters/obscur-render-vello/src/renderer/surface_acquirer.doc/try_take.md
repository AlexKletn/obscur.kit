The acquired image, if it has landed.

`None` means not yet, and the honest response to it is to skip the frame rather
than wait. The request stays in flight; the wake callback will bring the loop
back.
